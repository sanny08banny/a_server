use crate::db_client::db_client;
use axum::Json;
use fcm;
use hyper::StatusCode;
use serde_json::{json, Value};

pub async fn req_ride(det: Json<Value>) -> Result<StatusCode, StatusCode> {
	let det = det.0;
	send_notification(det, "Driver").await;
	Ok(StatusCode::OK)
}

pub async fn book_car(det: Json<Value>) -> Result<StatusCode, StatusCode> {
	let det = det.0;
	send_notification(det, "Owner").await;
	Ok(StatusCode::OK)
}

async fn send_notification(det: Value, category: &str) {
	let db = db_client().await;
	let client_id = det["client_id"].as_str().unwrap();
	let recepient = det["recepient_id"].as_str().unwrap();

	let client = fcm::Client::new();
	let mut details: Value = json!({});
	// get username from db
	let mut query = format!("SELECT user_name, user_phone FROM users WHERE user_id='{}'", client_id);
	let res = db.query(query.as_str(), &[]).await.unwrap();
	let user_name: String = res[0].get("user_name");
	let user_phone: String = res[0].get("user_phone");
	if category == "Driver" {
		details = json!(
		{
			"ride_id": client_id.to_owned()+"_"+recepient,
			"user_name": user_name,
			"user_phone": user_phone,
			"dest_lat": det["dest_lat"].as_f64().unwrap(),
			"dest_lon": det["dest_lon"].as_f64().unwrap(),
			"current_lat": det["current_lat"].as_f64().unwrap(),
			"current_lon": det["current_lon"].as_f64().unwrap(),
			"client_id": client_id,
		});
	} else if category == "Owner" {
		details = json!(
		{
			"booking_id": client_id.to_owned()+"_"+recepient,
			"user_name": user_name,
			"user_phone": user_phone,
			"car_id": det["car_id"].as_str().unwrap(),
			"client_id": client_id,
		});
	}
	query = format!("SELECT notification_token FROM users WHERE user_id='{}'", recepient);
	let res = db.query(query.as_str(), &[]).await.unwrap();
	let token: String = res[0].get("notification_token");

	let mut notification_builder = fcm::NotificationBuilder::new();
	if category == "Driver" {
		notification_builder.title("Ride request!");
		notification_builder.body("User 1 has requested a ride!");
		notification_builder.tag("Ride req");
	} else if category == "Owner" {
		notification_builder.title("Booking notification!");
		notification_builder.body("User 1 has booked your car!");
		notification_builder.tag("Booked car");
	}
	let notification = notification_builder.finalize();
	let mut message_builder = fcm::MessageBuilder::new(
		"AAAA1rzD5J4:APA91bEVDhK6QTL835XVuXPEEA0mbtV1q37zzZeTd0R7w2wHwyh-QyEjYP1CqZ2Jv6GKiSbuOrdLVi62TAThdyy4uPK4rYuphOLQPX_pfsx-l98jUmNPp6l_H7zCD_Jlq2i2-UZVlSXm",
		&token,
	);
	//  "c4JWkJpESg6I1q2irDFAbQ:APA91bGm01z1FVqLvwKr9qhkFauhSlBsN7PNbwnuQ7hjL_-yWTNnBffB5vs-IHePAW9UMQ7KNXl3T4KxxPs2JYKPK8SOa51N9wXPsNHX_Zvm-fB62r0A91x8eCbkxrcWBj3KR0Y0QKpv");
	message_builder.notification(notification);
	message_builder.data(&mut details);

	let response = client.send(message_builder.finalize()).await.unwrap();
	println!("Sent: {:?}", response);
}
