use axum::Json;
use base64::Engine;
use chrono::Local;
use serde_json::Value;

use crate::encryption_engine::CUSTOM_ENGINE;

#[derive(serde::Deserialize, serde::Serialize)]
struct Comment {
	id: String,
	user_name: String,
	title: String,
	comment: String,
	rating: f64,
}

#[derive(serde::Serialize)]
pub struct CarReview {
	owner_id: String,
	car_id: String,
	score: f32,
	comments: Vec<Comment>,
}

pub async fn get_review(ids: Json<Value>) -> Json<CarReview> {
	let ids = ids.0;
	let car_id = ids.get("car_id").unwrap().to_string();
	let owner_id = ids.get("owner_id").unwrap().to_string();

	// TODO: fetch review from DB
	let comment = Comment {
		id: "get from db".to_string(),
		user_name: "USER_NAME".to_string(),
		title: "This the title".to_string(),
		comment: "Awesome".to_string(),
		rating: 5.0,
	};

	Json(CarReview {
		owner_id,
		car_id,
		score: 3.5,
		comments: vec![comment],
	})
}

pub async fn create_review(rev: Json<Value>) {
	let rev = rev.0;
	let user_name = rev.get("user_id").unwrap().to_string();
	let title = rev.get("title").unwrap().to_string();
	let car_id = rev.get("car_id").unwrap().to_string();
	let comment = rev.get("comment").unwrap().to_string();
	let rating = rev.get("rating").unwrap().as_f64().unwrap();
	let owner_id = "get from db".to_string();
	let timestamp = Local::now().format("%Y%m%d%H%M%S").to_string();
	let input = format!("{}{}{}", car_id, owner_id, timestamp);
	let id = CUSTOM_ENGINE.encode(input);

	let rev = Comment {
		id,
		user_name,
		title,
		comment,
		rating,
	};
}
