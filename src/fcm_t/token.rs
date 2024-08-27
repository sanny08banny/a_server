use axum::extract::Path;
use axum::extract::State;
use hyper::StatusCode;

use crate::db_client::DbClient;

pub async fn update_token(db: State<DbClient>, Path((user_id, token)): Path<(String, String)>) -> StatusCode {
	match db.execute("UPDATE users SET notification_token=$1 WHERE user_id=$2", &[&token, &user_id]).await {
		Ok(_) => StatusCode::OK,
		Err(_) => StatusCode::INTERNAL_SERVER_ERROR,
	}
}
