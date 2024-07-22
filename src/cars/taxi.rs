use axum::body::Body;
use axum::extract::Path;
use axum::response::IntoResponse;
use axum::{extract::State, Json};
use base64::Engine;
use hyper::StatusCode;
use postgres::types::*;
use postgres_from_row::FromRow;
use serde_json::{json, Value};
use tokio::fs::File;
use tokio_util::io::ReaderStream;

use crate::fcm_t::fcm::{send_notification, start_ride_request, RideDetails};
use crate::{db_client::DbClient, encryption_engine};

#[derive(Debug, serde::Deserialize, serde::Serialize)]
pub enum TaxiCategory {
	Economy,
	X,
	Xl,
}

impl TaxiCategory {
	pub fn as_str(&self) -> &'static str {
		match self {
			TaxiCategory::Economy => "Economy",
			TaxiCategory::X => "X",
			TaxiCategory::Xl => "Xl",
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
			"X" => Ok(TaxiCategory::X),
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

	(StatusCode::OK, taxi_id)
}

pub async fn accept_ride_request(db: State<DbClient>, res: Json<Value>) -> StatusCode {
	let res = res.0;
	let client_id = res["client_id"].as_str().unwrap();
	let driver_id = res["driver_id"].as_str().unwrap();

	let Ok(res) = db.query_one("SELECT notification_token,user_name FROM users WHERE user_id=$1", &[&client_id]).await else {
		return StatusCode::INTERNAL_SERVER_ERROR;
	};
	let client_token: String = res.get("notification_token");

	let Ok(res) = db.query_one("SELECT plate_number,color,model FROM taxi WHERE user_id='$1'", &[&driver_id]).await else {
		return StatusCode::INTERNAL_SERVER_ERROR;
	};

	let details = json!({
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
start_ride_request(db, ride_details).await
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
		let driver_id: String = row.get("driver_id");
		let name: String = db.query_one("SELECT user_name FROM users WHERE user_id=$1", &[&driver_id]).await.unwrap().get("user_name");

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
		"DrivingLicense" => "driving_licence.png",
		"PSVLicense" => "psv_licence.png",
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
