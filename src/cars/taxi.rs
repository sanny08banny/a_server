use crate::{db_client::DbClient, ecryption_engine};
use crate::file_server::file_content;
use axum::{extract::State, Json};
use base64::Engine;
use hyper::{Body, StatusCode};
use postgres::Row;
use postgres_from_row::FromRow;
use serde_json::{json, Value};

#[derive(Debug, serde::Deserialize, FromRow, serde::Serialize)]
pub struct Taxi {
	pub driver_id: String,
	pub model: String,
	pub color: String,
	pub manufacturer: String,
	pub plate_number: String,
	pub category: String,
}

pub async fn init_taxi(db: State<DbClient>, taxi: Json<Taxi>) -> String {
	let taxi = taxi.0;
	let taxi_id = ecryption_engine::CUSTOM_ENGINE.encode(format!("{}{}{}{}", taxi.driver_id, taxi.plate_number, taxi.model, taxi.color));

	let mut statement = "INSERT INTO taxi (taxi_id,driver_id,model,color,plate_number,category,manufacturer,verified) VALUES ($1,$2,$3,$4,$5,$6,$7,$8)";
	db.execute(
		statement,
		&[&taxi_id, &taxi.driver_id, &taxi.model, &taxi.color, &taxi.plate_number, &taxi.category, &taxi.manufacturer,&false],
	)
	.await
	.unwrap();
	statement="INSERT INTO taxi_verifications (driver_id,inspection_report
	insurance,
	driving_licence,
	psv_licence,
	national_id) VALUES ($1,$2,$3,$4,$5,$6)";
	db.execute(
	statement,&[&taxi.driver_id,&false,&false,&false,&false,&false])
	.await
	.unwrap();

	taxi_id
}

#[derive(serde::Deserialize)]
pub struct TaxiDetailsReq {
	taxi_id: String,
}

pub async fn taxi_details(db: State<DbClient>, det: Json<TaxiDetailsReq>) -> Json<Option<Taxi>> {
	let taxi = db.query_opt("SELECT * FROM taxi WHERE taxi_id=$1", &[&det.taxi_id]).await.unwrap();

	match taxi {
		Some(t) => Json(Some(Taxi::from_row(&t))),
		None => Json(None),
	}
}

/* 
taxi_verifications table
driver_id
inspection_report
insurance
driving_licence
psv_licence
national_id
 */
// taxi table + verified column
async fn get_unverified_taxis(db: State<DbClient>)->Json<Vec<Row>>{
      let drivers=db.query("SELECT driver_id,email FROM taxi WHERE verified=$1", &[&false]).await.unwrap();
	  return Json(drivers);
}

// accessible to both the driver and admin
async fn get_unverified_documents(db: State<DbClient>,driver_id:String)->Value{
	let verification_documents=db.query_opt("SELECT * FROM taxi_verifications WHERE driver_id=$1", &[&driver_id]).await.unwrap();
	match  verification_documents {
		Some(row) => {
			let docs=vec!["national_id","insurance","driving_licence","psv_licence","inspection_report"];
			let mut unverified=Vec::new();
			for doc in docs{
				if !row.get::<_,bool>(doc){
					unverified.push(doc);
				}
			}
			// check if verification is complete and update status
			if unverified.len()==0{
				db.execute("UPDATE taxi SET verified=true WHERE driver_id=$1", &[&driver_id])
				.await
				.unwrap();
			}
            json!(unverified)
		},
		None => json!({"Status":"Driver not found"}),
	}
}

#[derive(serde::Deserialize)]
enum VerificationDocumentType {
	NationalId,
	Insurance,
	DrivingLicence,
	PSVLicence,
	InspectionReport
}

#[derive(serde::Deserialize)]
struct VerificationObject{
	driver_id:String,
	document_type:VerificationDocumentType
}

async fn get_unverified_document(req: VerificationObject)->Body{
	match req.document_type {	
		VerificationDocumentType::NationalId => Body::from(file_content(format!("images/taxi/{}/national_id_front.png",req.driver_id))),
		VerificationDocumentType::Insurance => Body::from(file_content(format!("images/taxi/{}/insurance.png",req.driver_id))),
		VerificationDocumentType::DrivingLicence => Body::from(file_content(format!("images/taxi/{}/driving_licence.png",req.driver_id))),
		VerificationDocumentType::PSVLicence => Body::from(file_content(format!("images/taxi/{}/psv_licence.png",req.driver_id))),
		VerificationDocumentType::InspectionReport => Body::from(file_content(format!("images/taxi/{}/inspection_report.png",req.driver_id))),
	}
}


async fn verify_document(db: State<DbClient>, req:VerificationObject)->StatusCode{
	let modified:u64;
	match req.document_type {
		VerificationDocumentType::NationalId => { modified=db.0.execute("UPDATE taxi_verifications SET national_id=true WHERE driver_id=$1", &[&req.driver_id]).await.unwrap()},
		VerificationDocumentType::Insurance => {modified=db.0.execute("UPDATE taxi_verifications SET insurance=true WHERE driver_id=$1", &[&req.driver_id]).await.unwrap()},
		VerificationDocumentType::DrivingLicence => { modified=db.0.execute("UPDATE taxi_verifications SET driving_licence=true WHERE driver_id=$1", &[&req.driver_id]).await.unwrap()},
		VerificationDocumentType::PSVLicence => { modified=db.0.execute("UPDATE taxi_verifications SET psv_licence=true WHERE driver_id=$1", &[&req.driver_id]).await.unwrap()},
		VerificationDocumentType::InspectionReport => { modified=db.0.execute("UPDATE taxi_verifications SET inspection_report=true WHERE driver_id=$1", &[&req.driver_id]).await.unwrap()},
	}
	if modified==1{
		StatusCode::OK
	}else {
		StatusCode::NOT_MODIFIED
	}
}