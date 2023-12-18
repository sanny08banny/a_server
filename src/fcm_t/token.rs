use axum::extract::Path;
use hyper::StatusCode;

use crate::db_client::db_client;

pub async fn update_token(h:Path<(String,String)>) -> Result<StatusCode, StatusCode> {
    let user_id = h.0.0;
    let token = h.0.1;
    let db = db_client().await;
    let query = format!("UPDATE users SET notification_token='{}' WHERE user_id='{}'", token, user_id);
    db.execute(query.as_str(), &[]).await.unwrap();
    Ok(StatusCode::OK)
}