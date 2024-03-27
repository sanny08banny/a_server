use axum::{extract::Path, Json};
use hyper::StatusCode;
use serde_json::Value;

use crate::db_client::db_client;

pub async fn update_token(h: Path<(String, String)>) -> Result<StatusCode, StatusCode> {
	let user_id = h.0 .0;
	let token = h.0 .1;
	let db = db_client().await;
	println!("{} {}", user_id, token);
	let query = format!("UPDATE users SET notification_token='{}' WHERE user_id='{}'", token, user_id);
	db.execute(query.as_str(), &[]).await.unwrap();
	Ok(StatusCode::OK)
}

pub async fn driver_response(res: Json<Value>) -> Result<StatusCode, StatusCode> {
	let res = res.0;
	let client_id = res["client_id"].as_str().unwrap();
	let status = res["status"].as_str().unwrap();
	println!("{} {}", client_id, status);
	return Ok(StatusCode::OK);
}
