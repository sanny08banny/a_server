use axum::Json;

use crate::db_client;

pub async fn query_token(uid: Json<String>) -> Json<f64> {
	let g = db_client().await;
	let uid = uid.0;
	let x = format!("SELECT tokens FROM users WHERE user_id='{}'", uid);
	let rows = g.query(x.as_str(), &[]).await.unwrap();
	let mut user_tokens = 0.00;
	for row in rows {
		user_tokens = row.get::<_, f64>("tokens");
	}
	axum::Json(user_tokens)
}
