use axum::Json;
use base64::Engine;
use serde_json::{json, Value};
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

pub async fn taxi_details(det:Json<Value>)->Json<Value>{
  let det = det.0;
  let taxi_id = det.get("taxi_id").unwrap().to_string();
  let q = format!("SELECT * FROM taxi WHERE taxi_id = '{}'",taxi_id);
  let g = db_client().await;
  let rows = g.query(&q, &[]).await.unwrap();
  let template = json!(
    {
      "driver_id": "",
      "model": "",
      "color": "",
      "plate_number": "",
      "manufacturer": "",
      "national_id_front": "national_id_front_filepath",
      "national_id_back": "national_id_back_filepath",
      "driving_license_front": "driving_license_front_filepath",
      "driving_license_back": "driving_license_back_filepath",
      "insurance": "insurance_filepath",
      "inspection_report": "inspection_report_filepath",
      "psv_license": "psv_licence_filepath",
      "taxi_images": ["imgpath1","imgpath2","imgpath3"],
    }
  );
  return Json(template);
}