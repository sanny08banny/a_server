use std::collections::HashMap;

use axum::body::Body;
use axum::extract::Path;
use axum::response::IntoResponse;
use axum::{extract::State, Json};
use base64::Engine;
use firebase_rs::Firebase;
use hyper::StatusCode;
use postgres::types::*;
use postgres_from_row::FromRow;
use serde_json::{json, Value};
use tokio::fs::File;
use tokio_util::io::ReaderStream;

use crate::fcm_t::fcm::{send_notification, start_notification};
use crate::users::UserType;
use crate::{db_client::DbClient, encryption_engine};

#[derive(Debug, serde::Deserialize, serde::Serialize)]
pub enum TaxiCategory {
	Economy,
	Classic,
	Xl,
	BodaBoda
}

impl TaxiCategory {
	pub fn as_str(&self) -> &'static str {
		match self {
			TaxiCategory::Economy => "Economy",
			TaxiCategory::Classic => "Classic",
			TaxiCategory::Xl => "Xl",
			TaxiCategory::BodaBoda=>"BodaBoda"
		}
	}
}

impl ToSql for TaxiCategory {
	fn to_sql(&self, ty: &Type, out: &mut bytes::BytesMut) -> Result<IsNull, Box<dyn std::error::Error + Sync + Send>>
	where
		Self: Sized,
	{
		self.as_str().to_sql(ty, out)
	}

	fn accepts(ty: &Type) -> bool
	where
		Self: Sized,
	{
		<&str as ToSql>::accepts(ty)
	}

	fn to_sql_checked(&self, ty: &Type, out: &mut bytes::BytesMut) -> Result<IsNull, Box<dyn std::error::Error + Sync + Send>> {
		self.as_str().to_sql_checked(ty, out)
	}
}

impl<'a> FromSql<'a> for TaxiCategory {
	fn from_sql(ty: &Type, raw: &'a [u8]) -> Result<Self, Box<dyn std::error::Error + Sync + Send>> {
		let data = <&str as FromSql>::from_sql(ty, raw)?;
		match data {
			"Economy" => Ok(TaxiCategory::Economy),
			"Xl" => Ok(TaxiCategory::Xl),
			"Classic" => Ok(TaxiCategory::Classic),
			"BodaBoda"=>Ok(TaxiCategory::BodaBoda),
			unknown => Err(format!("Unknown Taxi category: {}", unknown).into()),
		}
	}

	fn accepts(ty: &Type) -> bool {
		<&str as ToSql>::accepts(ty)
	}
}

#[derive(Debug, serde::Deserialize, FromRow, serde::Serialize)]
pub struct Taxi {
	pub driver_id: String,
	pub model: String,
	pub color: String,
	pub manufacturer: String,
	pub plate_number: String,
	pub category: TaxiCategory,
}


pub async fn init_taxi(db: State<DbClient>, taxi: Json<Taxi>) -> impl IntoResponse {
	let taxi = taxi.0;
	let taxi_id = encryption_engine::CUSTOM_ENGINE.encode(format!("{}{}{}{}", taxi.driver_id, taxi.plate_number, taxi.model, taxi.color));
	// debug
	if db.query_one("SELECT taxi_id WHERE driver_id=$1", &[&taxi.driver_id]).await.is_ok(){
		db.execute("DELETE taxi WHERE driver_id=$1", &[&taxi.driver_id]).await;
	}
	let statement = "INSERT INTO taxi (taxi_id,driver_id,model,color,plate_number,category,manufacturer,verified) VALUES ($1,$2,$3,$4,$5,$6,$7,$8)";
	if let Err(err) = db
		.execute(
			statement,
			&[&taxi_id, &taxi.driver_id, &taxi.model, &taxi.color, &taxi.plate_number, &taxi.category, &taxi.manufacturer, &false],
		)
		.await
	{
		return (StatusCode::INTERNAL_SERVER_ERROR, err.to_string());
	};

	let statement = "INSERT INTO taxi_verifications (driver_id, inspection_report, insurance, driving_license, psv_license, national_id) VALUES ($1,$2,$3,$4,$5,$6)";
	if let Err(err) = db.execute(statement, &[&taxi.driver_id, &false, &false, &false, &false, &false]).await {
		return (StatusCode::INTERNAL_SERVER_ERROR, err.to_string());
	};
	match db.execute("UPDATE users SET isdriver=true WHERE user_id=$1", &[&taxi.driver_id]).await {
		Ok(_) => (StatusCode::OK, taxi_id),
		Err(e) => (StatusCode::INTERNAL_SERVER_ERROR,e.to_string()),
	}
}

#[derive(serde::Serialize, serde::Deserialize, Debug)]
struct TaxiLocation{
driver_id:String,
latitude:f64,
longitude:f64,
orientation:f64,
seats:i32,
}

#[derive(serde::Serialize, serde::Deserialize, Debug)]
pub struct PricingDetails{
rider_id:String,
pick_up_latitude:f64,
pick_up_longitude:f64,
dest_latitude:f64,
dest_longitude:f64,
taxi_category:TaxiCategory,
}

#[derive(serde::Serialize, serde::Deserialize, Debug)]
pub struct RideDetails{
pricing_details:PricingDetails,
dest_name:String,
price:f64,
declined:Vec<String>,
pub iteration:i32,
phone_number:String
}


pub const EARTH_RADIUS: f64 = 6_366_707.0195;

pub fn great_circle_distance(a: (f64, f64), b: (f64, f64)) -> f64 {
	let lat1 = a.0.to_radians();
	let lon1 = a.1.to_radians();
	let lat2 = b.0.to_radians();
	let lon2 = b.1.to_radians();

	let delta_lon = lon2 - lon1;

	let central_angle = (lat1.sin() * lat2.sin() + lat1.cos() * lat2.cos() * delta_lon.cos()).acos();

	central_angle * EARTH_RADIUS
}

pub async fn taxi_price(ride_details: Json<PricingDetails>)->String{
let distance=great_circle_distance((ride_details.pick_up_latitude,ride_details.pick_up_longitude), (ride_details.dest_latitude,ride_details.pick_up_longitude))/1000.00;
let price:f64;
match ride_details.taxi_category {
	TaxiCategory::Economy => price=distance*50.00+120.00,
	TaxiCategory::Classic => price=distance*55.00+150.00,
	TaxiCategory::Xl => price=distance*65.00+200.00,
	TaxiCategory::BodaBoda=>price=distance*50.00+50.00,
}
price.round().to_string()
}

pub async fn reqest_ride(db: State<DbClient>,ride_details: Json<RideDetails>)->StatusCode {
	let ride_details=ride_details.0;
	start_ride_request(db, ride_details).await
}

pub async fn start_ride_request(db: State<DbClient>,ride_details: RideDetails)->StatusCode{
	let client_lat = ride_details.pricing_details.pick_up_latitude;
	let client_log = ride_details.pricing_details.pick_up_longitude;
	let mut closest_driver=String::new();
	let mut min_distance=0.00;
	let mut i=0;
	let mut skip=false;
    let firebase=Firebase::new("https://iris-59542-default-rtdb.firebaseio.com/").unwrap().at("taxis").at("available").at(ride_details.pricing_details.taxi_category.as_str());
    let base:HashMap<String,TaxiLocation>=firebase.get::<>().await.unwrap();
	for (_x,y) in base {
		for driver in &ride_details.declined {
			if driver==&y.driver_id{
				skip=true;
				break;
			}
		}
		if skip{
			continue;
		}
		let driver_lat = y.latitude;
		let driver_lon: f64 = y.longitude;
		let distance=great_circle_distance((client_lat,client_log), (driver_lat,driver_lon));
		if i==0{
			min_distance=distance;
		}else if distance<min_distance{
			min_distance=distance;
			closest_driver=y.driver_id;
		}
		if min_distance<=500.00{
			break;
		}else if ride_details.iteration >5 &&min_distance<=1500.00 {
			break;
		}
		i+=1;
	}
	if min_distance>1800.00{
		return StatusCode::NOT_FOUND;
	}
	let notification_details=json!({
		"sender_id":ride_details.pricing_details.rider_id,
		"recipient_id":closest_driver,
		"dest_lat":ride_details.pricing_details.dest_latitude,
		"dest_lon":ride_details.pricing_details.dest_longitude,
		"dest_name":ride_details.dest_name,
		"phone_number":ride_details.phone_number,
		"current_lat":ride_details.pricing_details.pick_up_latitude,
		"current_lon":ride_details.pricing_details.pick_up_longitude,
		"price":ride_details.price
	});
	start_notification(&db.0, notification_details, UserType::Driver).await
}


pub async fn accept_ride_request(db: State<DbClient>, res: Json<Value>) -> StatusCode {
	let res = res.0;
	let client_id = res["client_id"].as_str().unwrap();
	let driver_id = res["driver_id"].as_str().unwrap();

	let Ok(res) = db.query_one("SELECT notification_token,user_name FROM users WHERE user_id=$1", &[&client_id]).await else {
		return StatusCode::INTERNAL_SERVER_ERROR;
	};
	let client_token: String = res.get("notification_token");

	let Ok(res) = db.query_one("SELECT plate_number,color,model FROM taxi WHERE driver_id=$1", &[&driver_id]).await else {
		return StatusCode::INTERNAL_SERVER_ERROR;
	};
	let Ok(n) = db.query_one("SELECT user_name,user_phone FROM users WHERE user_id=$1", &[&driver_id]).await else {
		return StatusCode::INTERNAL_SERVER_ERROR;
	};
	let details = json!({
		"driver_name":n.get::<_,&str>(0),
		"phone_number":n.get::<_,&str>(1),
		"plate_number": res.get::<_,&str>("plate_number"),
		"color":res.get::<_,&str>("color"),
		"model":res.get::<_,&str>("model"),
	});

	send_notification(&client_token, details).await;
	StatusCode::OK
}


pub async fn decline_ride_request(db: State<DbClient>,ride_details:Json<RideDetails>){
let mut ride_details=ride_details.0;
ride_details.iteration+=1;
	start_ride_request(db, ride_details).await;
}

#[derive(serde::Deserialize)]
pub struct TaxiDetailsReq {
	taxi_id: String,
}

pub async fn taxi_details(db: State<DbClient>, det: Json<TaxiDetailsReq>) -> impl IntoResponse {
	match db.query_opt("SELECT * FROM taxi WHERE taxi_id=$1", &[&det.taxi_id]).await {
		Ok(taxi) => (StatusCode::OK, Json(taxi.map(|t| Taxi::from_row(&t)))),
		Err(_) => (StatusCode::INTERNAL_SERVER_ERROR, Json(None)),
	}
}

#[derive(serde::Serialize)]
pub struct UnverifiedDriver {
	name: String,
	driver_id: String,
}

pub async fn get_unverified_taxis(db: State<DbClient>) -> Json<Vec<UnverifiedDriver>> {
	let rows = db.query("SELECT driver_id FROM taxi WHERE verified=$1", &[&false]).await.unwrap();
	let mut drivers = Vec::with_capacity(rows.len());
	for row in rows {
		let driver_id: String = row.get(0);
		let name:String = db.query_one("SELECT user_name FROM users WHERE user_id=$1", &[&driver_id]).await.unwrap().get(0);
		drivers.push(UnverifiedDriver { name, driver_id })
	}

	Json(drivers)
}

/*
taxi_verifications table
driver_id
inspection_report
insurance
driving_license
psv_licence
national_id
 */
// taxi table + verified column

// accessible to both the driver and admin
pub async fn get_unverified_documents(db: State<DbClient>, Path(driver_id): Path<String>) -> impl IntoResponse {
	let verification_documents = db.query_opt("SELECT * FROM taxi_verifications WHERE driver_id=$1", &[&driver_id]).await.unwrap();

	match verification_documents {
		Some(row) => {
			let required = ["national_id", "insurance", "driving_license", "psv_license", "inspection_report"];
			let missing = required.into_iter().filter(|r| !row.get::<_, bool>(r)).collect::<Vec<_>>();

			// check if verification is complete and update status
			if missing.is_empty() {
				db.execute("UPDATE taxi SET verified=true WHERE driver_id=$1", &[&driver_id]).await.unwrap();
			}

			(StatusCode::OK, Json(missing))
		}
		None => (StatusCode::NOT_FOUND, Json(vec![])),
	}
}

#[derive(serde::Deserialize, serde::Serialize)]
enum VerificationDocumentType {
	NationalId,
	Insurance,
	DrivingLicense,
	PSVLicense,
	InspectionReport,
}

pub async fn get_unverified_document(Path((driver_id, document_type)): Path<(String, String)>) -> impl IntoResponse {
	let path = match document_type.as_str() {
		"NationalId" => "national_id_front.png",
		"Insurance" => "insurance.png",
		"DrivingLicense" => "driving_license.png",
		"PSVLicense" => "psv_license.png",
		"InspectionReport" => "inspection_report.png",
		_ => return (StatusCode::BAD_REQUEST, Body::empty()),
	};

	let path = format!("images/taxi/{}/{}", driver_id, path);
	let Ok(file) = File::open(path).await else {
		return (StatusCode::NOT_FOUND, Body::empty());
	};

	let stream = ReaderStream::new(file);
	(StatusCode::OK, Body::from_stream(stream))
}

pub async fn verify_document(db: State<DbClient>, Path((driver_id, document_type)): Path<(String, String)>) -> StatusCode {
	let column = match document_type.as_str() {
		"NationalId" => "national_id",
		"Insurance" => "insurance",
		"DrivingLicense" => "driving_license",
		"PSVLicense" => "psv_license",
		"InspectionReport" => "inspection_report",
		_ => return StatusCode::BAD_REQUEST,
	};

	let query = format!("UPDATE taxi_verifications SET {}=true WHERE driver_id=$1", column);
	match db.0.execute(&query, &[&driver_id]).await {
		Ok(_) => StatusCode::OK,
		_ => StatusCode::INTERNAL_SERVER_ERROR,
	}
}
