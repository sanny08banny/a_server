use axum::{extract::State, Json};

use crate::db_client::DbClient;

pub async fn query_token(db: State<DbClient>, uid: Json<String>) -> Json<f64> {
	let uid = uid.0;
	let x = format!("SELECT tokens FROM users WHERE user_id='{}'", uid);
	let rows = db.query(x.as_str(), &[]).await.unwrap();
	let mut user_tokens = 0.00;
	for row in rows {
		user_tokens = row.get::<_, f64>("tokens");
	}
	axum::Json(user_tokens)
}
