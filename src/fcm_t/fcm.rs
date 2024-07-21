use crate::db_client::DbClient;
use axum::{extract::State, Json};
use fcm;
use serde_json::{json, Value};

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

pub async fn req_ride(db: State<DbClient>, details: Json<Vec<Value>>) {
	let mut closest_driver: usize = 0;
	let mut min_distance=0.00;
	let mut i=0;
	for detail in details.0.iter().cloned() {
		let client_lat = detail["current_lat"].as_f64().unwrap();
		let client_log = detail["current_lon"].as_f64().unwrap();
		let driver_lat = 0.00;
		let driver_lon: f64 = 0.00;
		let distance=great_circle_distance((client_lat,client_log), (driver_lat,driver_lon));
		if i==0{
			min_distance=distance;
		}else if distance<min_distance{
			min_distance=distance;
			closest_driver=i;
		}
		i+=1;
	}
	start_notification(&db.0, details.0[closest_driver].clone(), "Driver").await;
}

pub async fn book_car(db: State<DbClient>, detail: Json<Value>) {
	start_notification(&db.0, detail.0, "Owner").await;
}

pub async fn ride_request_status(db: State<DbClient>, detail: Json<Value>) {
	start_notification(&db.0, detail.0, "Normal").await;
}

pub async fn book_request_status(db: State<DbClient>, detail: Json<Value>) {
	start_notification(&db.0, detail.0, "Normal").await;
}

async fn start_notification(db: &DbClient, det: Value, category: &str) {
	let sender_id = det["sender_id"].as_str().unwrap();
	let recipient = det["recipient_id"].as_str().unwrap();

	// get username from db
	let res = db.query("SELECT user_name,user_phone FROM users WHERE user_id=$1", &[&sender_id]).await.unwrap();
	let user_name: String = res[0].get("user_name");

	let details = if category == "Driver" {
		json!(
		{
			"ride_id": sender_id.to_owned()+"_"+recipient,
			"user_name": user_name,
			"user_phone": det["phone_number"].as_str().unwrap(),
			"dest_lat": det["dest_lat"].as_f64().unwrap(),
			"dest_lon": det["dest_lon"].as_f64().unwrap(),
			"current_lat": det["current_lat"].as_f64().unwrap(),
			"current_lon": det["current_lon"].as_f64().unwrap(),
			"sender_id": sender_id,
		})
	} else if category == "Owner" {
		json!(
		{
			"booking_id": sender_id.to_owned()+"_"+recipient,
			"user_name": user_name,
			// "user_phone": user_phone,
			"car_id": det["car_id"].as_str().unwrap(),
			"sender_id": sender_id,
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

	let res = db.query("SELECT notification_token FROM users WHERE user_id=$1", &[&recipient]).await.unwrap();
	let token: String = res[0].get("notification_token");
	println!("recipient notification token: {:?}", token);
	send_notification(token.as_str(), details).await;
}

pub async fn send_notification(token: &str, mut details: Value) {
	let client = fcm::Client::new();
	let notification_builder = fcm::NotificationBuilder::new();
	let notification = notification_builder.finalize();
	let mut message_builder = fcm::MessageBuilder::new(option_env!("NOTIFICATION_API_KEY").expect("NOTIFICATION_API_KEY not set, unable to send notifications"), token);
	message_builder.notification(notification);
	message_builder.data(&mut details).unwrap();

	let response = client.send(message_builder.finalize()).await.unwrap();
	println!("Sent: {:?}", response);
}
