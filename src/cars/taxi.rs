use crate::{db_client, ecryption_engine};
use axum::Json;
use base64::Engine;
use serde_json::{json, Value};

pub async fn init_taxi(det: Json<Value>) -> Json<String> {
	let det = det.0;
	let driver_id = det.get("user_id").unwrap().to_string().replace("\"", "");
	let model = det.get("model").unwrap().to_string().replace("\"", "");
	let color = det.get("color").unwrap().to_string().replace("\"", "");
	let manufacturer = det.get("manufacturer").unwrap().to_string().replace("\"", "");
	let plate_number = det.get("plate_number").unwrap().to_string().replace("\"", "");
	let category = det.get("category").unwrap().to_string().replace("\"", "");
	let taxi_id = ecryption_engine::CUSTOM_ENGINE.encode(format!("{}{}{}{}", driver_id, plate_number, model, color));
	let q = format!(
		"
  INSERT INTO taxi 
  (taxi_id,driver_id,model,color,plate_number,category,manufacturer)
   VALUES ('{taxi_id}','{driver_id}','{model}','{color}','{plate_number}','{category}','{manufacturer}')",
	);
	let g = db_client().await;
	g.execute(&q, &[]).await.unwrap();
	return Json(taxi_id);
}

pub async fn taxi_details(det: Json<Value>) -> Json<Value> {
	let det = det.0;
	// "\"IjciIktESiA3MjZIIiJDb3JvbGxhIiItMTY3NzcyMTYi\"" to remove the escape characters

	let taxi_id = det.get("taxi_id").unwrap().to_string().replace("\"", "");
	println!("taxi_id {:?}", taxi_id);
	// let q = format!("SELECT * FROM taxi WHERE taxi_id='{}'",taxi_id);
	let g = db_client().await;
	let rows = g.query("SELECT * FROM taxi WHERE taxi_id=$1", &[&taxi_id]).await.unwrap();
	if rows.is_empty() {
		return Json(json!({"error": "taxi not found"}));
	}
	let driver_id: String = rows[0].get("driver_id");
	let category: String = rows[0].get("category");
	let model: String = rows[0].get("model");
	let color: String = rows[0].get("color");
	let plate_number: String = rows[0].get("plate_number");
	let manufacturer: String = rows[0].get("manufacturer");
	let taxi_images: Vec<String> = rows[0].get("image_paths");
	let res_body = json!(
	  {
		"driver_id": driver_id,
		"model": model,
		"category": category,
		"color": color,
		"plate_number": plate_number,
		"manufacturer": manufacturer,
		"taxi_images": taxi_images,
	  }
	);
	return Json(res_body);
}
