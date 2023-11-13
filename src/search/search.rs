use axum::Json;
use serde_json::Value;
use crate::db_client::db_client;

async fn search(keyword:Json<String>)->Json<Value>{
let keyword=keyword.0;
let g= db_client().await;

todo!()
}