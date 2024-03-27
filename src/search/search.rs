use std::collections::BTreeMap;

use crate::{db_client::db_client, Car};
use axum::Json;
use levenshtein;
use serde_json::Value;

async fn cars(search_res_map: BTreeMap<String, String>) -> Vec<Car> {
	let g = db_client().await;
	let mut x = Vec::new();
	for (key, value) in search_res_map {
		let f = format!("SELECT * FROM car WHERE {}='{}'", &value, key);
		let rows = g.query(f.as_str(), &[]).await.unwrap_or_else(|_| panic!("Error on query"));
		for row in rows {
			let owner_id: String = row.get::<_, String>("owner_id");
			let car_id: String = row.get::<_, String>("car_id");
			let model: String = row.get::<_, String>("model");
			let location: String = row.get::<_, String>("location");
			let description: String = row.get::<_, String>("description");
			let daily_amount: f64 = row.get::<_, f64>("daily_amount");
			let daily_downpayment_amt: f64 = row.get::<_, f64>("daily_downpayment_amt");
			let car_images: Vec<String> = row.get::<_, Vec<String>>("car_images");
			let available: bool = row.get::<_, bool>("available");
			let car = Car {
				car_images,
				model,
				car_id,
				owner_id,
				location,
				description,
				amount: daily_amount,
				downpayment_amt: daily_downpayment_amt,
				available,
			};
			x.push(car);
		}
	}
	x
}

pub async fn search(keyword: Json<String>) -> Json<Vec<Car>> {
	let keyword = keyword.0;

	let g = db_client().await;
	let mut location: Vec<String> = Vec::new();
	let mut model: Vec<String> = Vec::new();
	let mut description: Vec<String> = Vec::new();
	for row in g.query("SELECT * FROM car", &[]).await.unwrap_or_else(|_| panic!("Error on query")) {
		location.push(row.get::<_, String>("location"));
		model.push(row.get::<_, String>("model"));
		description.push(row.get::<_, String>("description"));
	}
	let mut name_result: Vec<String> = Vec::new();
	let mut model_result: Vec<String> = Vec::new();
	let mut description_result: Vec<String> = Vec::new();
	for i in 0..location.len() {
		if levenshtein::levenshtein(&location[i], &keyword) <= 2 {
			name_result.push(location[i].clone());
		}
		if levenshtein::levenshtein(&model[i], &keyword) <= 2 {
			model_result.push(model[i].clone());
		}
		if levenshtein::levenshtein(&description[i], &keyword) <= 2 {
			description_result.push(description[i].clone());
		}
	}
	let mut result: BTreeMap<String, String> = BTreeMap::new();
	for i in 0..name_result.len() {
		result.insert(name_result[i].clone(), "location".to_string());
	}
	for i in 0..model_result.len() {
		result.insert(model_result[i].clone(), "model".to_string());
	}
	for i in 0..description_result.len() {
		result.insert(description_result[i].clone(), "description".to_string());
	}
	let result = Json(cars(result).await);
	result
}
