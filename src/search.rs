use crate::{db_client::DbClient, Car};
use axum::{extract::State, Json};
use postgres_from_row::FromRow;

async fn cars(db: &DbClient, search_params: Vec<(String, String)>) -> Vec<Car> {
	let mut x = Vec::new();

	for (key, value) in search_params {
		let rows = db.query("SELECT * FROM car WHERE $1=$2", &[&key, &value]).await.unwrap();
		let iter = rows.iter().map(Car::from_row);

		x.extend(iter)
	}
	x
}

pub async fn search(db: State<DbClient>, keyword: Json<String>) -> Json<Vec<Car>> {
	let keyword = keyword.0;

	let mut location: Vec<String> = Vec::new();
	let mut model: Vec<String> = Vec::new();
	let mut description: Vec<String> = Vec::new();
	for row in db.query("SELECT * FROM car", &[]).await.expect("Unable to process query") {
		location.push(row.get("location"));
		model.push(row.get("model"));
		description.push(row.get("description"));
	}
	let mut names: Vec<String> = Vec::new();
	let mut models: Vec<String> = Vec::new();
	let mut descriptions: Vec<String> = Vec::new();
	for i in 0..location.len() {
		if levenshtein::levenshtein(&location[i], &keyword) <= 2 {
			names.push(location[i].clone());
		}
		if levenshtein::levenshtein(&model[i], &keyword) <= 2 {
			models.push(model[i].clone());
		}
		if levenshtein::levenshtein(&description[i], &keyword) <= 2 {
			descriptions.push(description[i].clone());
		}
	}

	let mut result: Vec<(String, String)> = Vec::new();

	for name in names.iter().cloned() {
		result.push((name, "location".to_string()));
	}

	for model in models.iter().cloned() {
		result.push((model, "model".to_string()));
	}

	for description in descriptions.iter().cloned() {
		result.push((description, "description".to_string()));
	}

	Json(cars(&db.0, result).await)
}
