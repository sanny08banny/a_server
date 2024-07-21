use axum::extract::State;
use axum::{extract::Path, Json};
use hyper::StatusCode;
use serde_json::{json, Value};

use crate::db_client::DbClient;
use crate::fcm_t::fcm::send_notification;

pub async fn update_token(db: State<DbClient>, Path((user_id, token)): Path<(String, String)>) -> StatusCode {
	match db.execute("UPDATE users SET notification_token=$1 WHERE user_id=$2", &[&token, &user_id]).await {
		Ok(_) => StatusCode::OK,
		Err(_) => StatusCode::INTERNAL_SERVER_ERROR,
	}
}

pub async fn driver_response(db: State<DbClient>, res: Json<Value>) -> StatusCode {
	let res = res.0;
	let client_id = res["client_id"].as_str().unwrap();
	let driver_id = res["driver_id"].as_str().unwrap();

	let Ok(res) = db.query_one("SELECT notification_token,user_name FROM users WHERE user_id=$1", &[&client_id]).await else {
		return StatusCode::INTERNAL_SERVER_ERROR;
	};
	let client_token: String = res.get("notification_token");

	let Ok(res) = db.query_one("SELECT plate_number,color,model FROM taxi WHERE user_id='$1'", &[&driver_id]).await else {
		return StatusCode::INTERNAL_SERVER_ERROR;
	};1

	let details = json!({
		"plate_number": res.get::<_,&str>("plate_number"),
		"color":res.get::<_,&str>("color"),
		"model":res.get::<_,&str>("model"),
	});

	send_notification(&client_token, details).await;
	StatusCode::OK
}
