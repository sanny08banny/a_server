use crate::{db_client::DbClient, ecryption_engine};
use axum::{extract::State, Json};
use base64::Engine;
use postgres_from_row::FromRow;

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

	let statement = "INSERT INTO taxi (taxi_id,driver_id,model,color,plate_number,category,manufacturer) VALUES ('$1','$2','$3','$4','$5','$6','$7')";
	db.execute(
		statement,
		&[&taxi_id, &taxi.driver_id, &taxi.model, &taxi.color, &taxi.plate_number, &taxi.category, &taxi.manufacturer],
	)
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
