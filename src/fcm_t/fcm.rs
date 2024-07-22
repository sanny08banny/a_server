use std::collections::HashMap;

use crate::{cars::taxi::TaxiCategory, db_client::DbClient};
use axum::{extract::State, Json};
use fcm;
use serde_json::{json, Value};
use firebase_rs::*;

#[derive(serde::Serialize, serde::Deserialize, Debug)]
struct TaxiLocation{
driver_id:String,
latitude:f64,
longitude:f64,
orientation:f64,
seats:i32,
}

#[derive(serde::Serialize, serde::Deserialize, Debug)]
pub struct RideDetails{
rider_id:String,
pick_up_latitude:f64,
pick_up_longitude:f64,
dest_latitude:f64,
dest_longitude:i32,
taxi_category:TaxiCategory,
price:i32,
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

pub async fn req_ride(db: State<DbClient>,ride_details: Json<RideDetails>) {
	let ride_details=ride_details.0;
	start_ride_request(db, ride_details).await
}

pub async fn start_ride_request(db: State<DbClient>,ride_details: RideDetails){
	let client_lat = ride_details.pick_up_latitude;
	let client_log = ride_details.pick_up_longitude;
	let mut closest_driver=String::new();
	let mut min_distance=0.00;
	let mut i=0;
    let firebase=Firebase::new("https://iris-59542-default-rtdb.firebaseio.com/").unwrap().at("taxis").at("available").at(ride_details.taxi_category.as_str());
    let base:HashMap<String,TaxiLocation>=firebase.get::<>().await.unwrap();
	for (_x,y) in base {
		for driver in &ride_details.declined {
			if driver==&y.driver_id{
				continue;
			}
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
	let notification_details=json!({
		"sender_id":ride_details.rider_id,
		"recipient_id":closest_driver,
		"dest_lat":ride_details.dest_latitude,
		"dest_lon":ride_details.dest_longitude,
		"phone_number":ride_details.phone_number,
		"current_lat":ride_details.pick_up_latitude,
		"current_lon":ride_details.pick_up_longitude,
		"price":ride_details.price
	});
	start_notification(&db.0, notification_details, "Driver").await;
}

pub async fn book_car(db: State<DbClient>, detail: Json<Value>) {
	start_notification(&db.0, detail.0, "Owner").await;
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
			"price":det["price"].as_f64().unwrap(),
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
