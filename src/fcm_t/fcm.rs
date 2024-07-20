use crate::db_client::DbClient;
use axum::{extract::State, Json};
use fcm;
use hyper::StatusCode;
use serde_json::{json, Value};

pub async fn req_ride(db: State<DbClient>, details: Json<Vec<Value>>) -> Result<StatusCode, StatusCode> {
	for detail in details.0.iter().cloned() {
		start_notification(&db.0, detail, "Driver").await;
	}
	Ok(StatusCode::OK)
}

pub async fn book_car(db: State<DbClient>, detail: Json<Value>) -> Result<StatusCode, StatusCode> {
	start_notification(&db.0, detail.0, "Owner").await;
	Ok(StatusCode::OK)
}

pub async fn ride_request_status(db: State<DbClient>, detail: Json<Value>) -> Result<StatusCode, StatusCode> {
	start_notification(&db.0, detail.0, "Normal").await;
	Ok(StatusCode::OK)
}

pub async fn book_request_status(db: State<DbClient>, detail: Json<Value>) {
	start_notification(&db.0, detail.0, "Normal").await;
}

async fn start_notification(db: &DbClient, det: Value, category: &str) {
	let client_id = det["client_id"].as_str().unwrap();
	let recepient = det["recepient_id"].as_str().unwrap();

	// get username from db
	let query = format!("SELECT user_name, user_phone FROM users WHERE user_id='{}'", client_id);
	let res = db.query(query.as_str(), &[]).await.unwrap();
	let user_name: String = res[0].get("user_name");
	// let user_phone: String = res[0].get("user_phone");

	// query = format!("SELECT notification_token FROM users WHERE user_id='{}'", client_id);
	// let res = db.query(query.as_str(), &[]).await.unwrap();
	// let client_token: String = res[0].get("notification_token");

	let details = if category == "Driver" {
		json!(
		{
			"ride_id": client_id.to_owned()+"_"+recepient,
			"user_name": user_name,
			// "client_token": client_token,
			// "user_phone": user_phone,
			"dest_lat": det["dest_lat"].as_f64().unwrap(),
			"dest_lon": det["dest_lon"].as_f64().unwrap(),
			"current_lat": det["current_lat"].as_f64().unwrap(),
			"current_lon": det["current_lon"].as_f64().unwrap(),
			"client_id": client_id,
		})
	} else if category == "Owner" {
		json!(
		{
			"booking_id": client_id.to_owned()+"_"+recepient,
			"user_name": user_name,
			// "user_phone": user_phone,
			"car_id": det["car_id"].as_str().unwrap(),
			"client_id": client_id,
		})
	} else if category == "Normal" && det["status"].as_str().unwrap() == "accepted" {
		json!({
			"status":"accepted"
		})
	} else {
		json!({
			"status":"rejected"
		})
	};

	let query = format!("SELECT notification_token FROM users WHERE user_id='{}'", recepient);
	let res = db.query(query.as_str(), &[]).await.unwrap();
	let token: String = res[0].get("notification_token");
	println!("recepient notification token: {:?}", token);
	send_notification(category, user_name.as_str(), token.as_str(), details).await;
}

pub async fn send_notification(_category: &str, _user_name: &str, token: &str, mut details: Value) {
	let client = fcm::Client::new();
	let notification_builder = fcm::NotificationBuilder::new();
	let notification = notification_builder.finalize();
	let mut message_builder = fcm::MessageBuilder::new(option_env!("NOTIFICATION_API_KEY").expect("NOTIFICATION_API_KEY not set, unable to send notifications"), token);
	message_builder.notification(notification);
	message_builder.data(&mut details).unwrap();

	let response = client.send(message_builder.finalize()).await.unwrap();
	println!("Sent: {:?}", response);
}
