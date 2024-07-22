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
