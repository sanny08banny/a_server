use crate::{db_client::DbClient, users::UserType};
use fcm;
use hyper::StatusCode;
use serde_json::{json, Value};


pub async fn start_notification(db: &DbClient, det: Value, category: UserType) -> StatusCode {
	let sender_id = det["sender_id"].as_str().unwrap();
	let recipient = det["recipient_id"].as_str().unwrap();

	// get username from db
	let res = db.query("SELECT user_name,user_phone FROM users WHERE user_id=$1", &[&sender_id]).await.unwrap();
	let user_name: String = res[0].get("user_name");

	let details = match category {
		UserType::Driver => {
			json!(
			{
				"ride_id": sender_id.to_owned()+"_"+recipient,
				"user_name": user_name,
				"user_phone": det["phone_number"],
				"dest_name":det["dest_name"],
				"dest_lat": det["dest_lat"],
				"dest_lon": det["dest_lon"],
				"current_lat": det["current_lat"],
				"current_lon": det["current_lon"],
				"price":det["price"],
				"sender_id": sender_id,
			})
		}
		UserType::Owner => {
			json!(
			{
				"booking_id": sender_id.to_owned()+"_"+recipient,
				"user_name": user_name,
				// "user_phone": user_phone,
				"car_id": det["car_id"].as_str().unwrap(),
				"sender_id": sender_id,
			})
		}
		UserType::Rider => {
			if det["status"].as_str().unwrap() == "accepted" {
				json!({
					"status":"accepted"
				})
			} else {
				json!({
					"status":"rejected"
				})
			}
		}
		UserType::Admin => todo!(),
	};
	println!("recipient= {}", recipient);
	let Ok(res) = db.query_one("SELECT notification_token FROM users WHERE user_id=$1", &[&recipient]).await else {
		return StatusCode::INTERNAL_SERVER_ERROR;
	};
	let Ok(token) = res.try_get::<_, &str>("notification_token") else {
		println!("notification token not found");
		return StatusCode::NOT_FOUND;
	};
	println!("recipient notification token: {:?}", token);
	send_notification(token, details).await;
	return StatusCode::OK;
}

pub async fn send_notification(token: &str, mut details: Value) {
	let client = fcm::Client::new();
	let notification_builder = fcm::NotificationBuilder::new();
	let notification = notification_builder.finalize();
	let mut message_builder = fcm::MessageBuilder::new(option_env!("NOTIFICATION_API_KEY").expect("NOTIFICATION_API_KEY not set, unable to send notifications"), token);
	message_builder.notification(notification);
	message_builder.data(&mut details).unwrap();
	let message= message_builder.finalize();

	let response = client.send(message).await.unwrap();
	println!("Sent: {:?}", response);
}
