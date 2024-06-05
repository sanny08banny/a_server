use axum::Json;
use base64::Engine;
use serde_json::Value;
use crate::{db_client, ecryption_engine};

pub async fn init_taxi(det:Json<Value>)->Json<String>
{
  let det=det.0;
  let driver_id = det.get("user_id").unwrap().to_string();
  let model=det.get("model").unwrap().to_string();
  let color=det.get("color").unwrap().to_string();
  let manufacturer=det.get("manufacturer").unwrap().to_string();
  let plate_number=det.get("plate_number").unwrap().to_string();
  let category = det.get("category").unwrap().to_string();
  let taxi_id=ecryption_engine::CUSTOM_ENGINE.encode(format!("{}{}{}{}",driver_id,plate_number,model,color));
  let q = format!("
  INSERT INTO taxi 
  (taxi_id,driver_id,model,color,plate_number,category,manufacturer)
   VALUES ('{taxi_id}','{driver_id}','{model}','{color}','{plate_number}','{category}','{manufacturer}')",);
  let g = db_client().await;
  g.execute(&q, &[]).await.unwrap();
  return  Json(taxi_id);
}
