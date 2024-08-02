use std::{fs, io::Write};

use crate::{db_client::DbClient, fcm_t::fcm::book_request_status};
use axum::{
	extract::{multipart::Field, Multipart, State},
	Json,
};
use hyper::StatusCode;
use postgres_from_row::FromRow;
use serde_json::{json, Value};

#[derive(serde::Deserialize, serde::Serialize, FromRow)]
pub struct Car {
	pub car_images: Vec<String>,
	pub model: String,
	pub car_id: String,
	pub owner_id: String,
	pub location: String,
	pub description: String,
	pub amount: f64,
	pub downpayment_amt: f64,
	pub available: bool,
}

#[derive(serde::Deserialize, Debug)]
pub struct BookingRequest {
	user_id: String,
	car_id: String,
	owner_id: String,
	description: BookingDetailsDesc,
}

#[derive(serde::Deserialize, Debug)]
#[serde(rename = "camelCase")]
#[non_exhaustive]
pub enum BookingDetailsDesc {
	Book,
	Unbook,
}

pub async fn get_cars(db: State<DbClient>) -> Json<Vec<Car>> {
	let rows = db.query("SELECT * FROM car", &[]).await.unwrap();
	let cars = rows.iter().map(Car::from_row).collect();
	Json(cars)
}

pub async fn accept_book(db: State<DbClient>, details: Json<BookingRequest>) -> StatusCode {
	let request = details.0;
	let mut details = json!({
		"client_id":request.owner_id,
		"recipient_id":request.user_id,
	});

	match request.description {
		BookingDetailsDesc::Book => {
			let y = format!("SELECT booking_tokens FROM car WHERE car_id='{}'", request.car_id);
			let rows = db.query(y.as_str(), &[]).await.unwrap();
			let mut booking_tokens = 0.00;
			for row in rows {
				booking_tokens = row.get::<_, f64>("booking_tokens");
			}
			let x = format!("SELECT tokens FROM users WHERE user_id='{}'", request.user_id);
			let rows = db.query(x.as_str(), &[]).await.unwrap();
			let mut user_tokens = 0.00;
			for row in rows {
				user_tokens = row.get::<_, f64>("tokens");
			}
			if user_tokens < booking_tokens {
				return StatusCode::EXPECTATION_FAILED;
			}
			let new_user_tokens = user_tokens - booking_tokens;
			let x = format!("UPDATE users SET tokens='{}' WHERE user_id='{}'", new_user_tokens, request.user_id);
			db.execute(x.as_str(), &[]).await.unwrap();
			details["status"] = Value::String("accepted".to_string());
			book_request_status(db, Json(details)).await;

			StatusCode::OK
		}
		BookingDetailsDesc::Unbook => {
			details["status"] = Value::String("accepted".to_string());
			book_request_status(db, Json(details)).await;

			StatusCode::OK
		}
	}
}

pub async fn multi_upload(db: State<DbClient>, mut multipart: Multipart) -> StatusCode {
	let mut user_id = String::new();
	let mut car_id = String::new();
	let mut model = String::new();
	let mut location = String::new();
	let mut description = String::new();
	let mut daily_price = String::new();
	let mut daily_down_payment = String::new();
	let mut available = true;
	let mut images: Vec<String> = Vec::new();
	let mut file_path = String::new();
	let mut index = 0;
	let mut category = String::new();
	let mut columns: Vec<String> = Vec::with_capacity(5);

	loop {
		let next = multipart.next_field().await;

		match next {
			Ok(Some(field)) => {
				let name = field.name().unwrap().to_string();
				println!("{:?}", name);
				match name.as_str() {
					"category" => {
						category = field.text().await.unwrap().replace('"', "");
		
						if category == "taxi" {
							file_path = "images/taxi/".to_owned();
						} else if category == "car_hire" {
							file_path = "images/car_hire/".to_owned();
						}
					}
					"user_id" => {
						user_id = field.text().await.unwrap().replace('"', "");
						file_path = file_path + &user_id + "/";
					}
					"car_id" => {
						car_id = field.text().await.unwrap().replace('"', "");
						if category == "hire" {
							file_path = file_path + &car_id + "/";
						}
						match fs::create_dir_all(&file_path) {
							Ok(_) => {}
							Err(e) => {
								println!("failed to create directories {}", e)
							}
						}
					}
					"model" => {
						model = field.text().await.unwrap().replace('"', "");
					}
					"location" => {
						location = field.text().await.unwrap().replace('"', "");
					}
					"description" => {
						description = field.text().await.unwrap().replace('"', "");
					}
					"daily_price" => {
						daily_price = field.text().await.unwrap().replace('"', "");
					}
					"daily_down_payment" => {
						daily_down_payment = field.text().await.unwrap().replace('"', "");
					}
					"available" => {
						available = field.text().await.unwrap().parse().unwrap();
					}
					"inspection_report_expiry" => {
						print!("inspection_report_expiry {:?}", field.text().await.unwrap());
					}
					"inspection_report" => {
						save_file(&file_path, "inspection_report.png", &field.bytes().await.unwrap());
						columns.push(String::from("inspection_report"));
					}
					"insurance_payment_plan" => {
						print!("insurance_payment_plan, {:?}", field.text().await.unwrap());
					}
					"insurance_expiry" => {
						print!("insurance_expiry, {:?}", field.text().await.unwrap());
					}
					"insurance" => {
						save_file(&file_path, "insurance.png", &field.bytes().await.unwrap());
						columns.push(String::from("insurance"));
					}
					"driving_license_front" => {
						save_file(&file_path, "driving_license_front.png", &field.bytes().await.unwrap());
					}
					"driving_license_back" => {
						save_file(&file_path, "driving_license_back.png", &field.bytes().await.unwrap());
						columns.push(String::from("driving_license"));
					}
					"psv_license" => {
						save_file(&file_path, "psv_license.png", &field.bytes().await.unwrap());
						columns.push(String::from("psv_license"));
					}
					"national_id_front" => {
						save_file(&file_path, "national_id_front.png", &field.bytes().await.unwrap());
					}
					"national_id_back" => {
						save_file(&file_path, "national_id_back.png", &field.bytes().await.unwrap());
						columns.push(String::from("national_id"));
					}
					_ => {
						let img_name = format!("img_{}.{}", index, "png");
						let img = image::load_from_memory(&field.bytes().await.unwrap()).unwrap();
						file_path.push_str(&img_name);
						match img.save(&file_path) {
							Ok(_) => {
								index += 1;
							}
							Err(e) => {
								println!("Failed to save image: {}", e)
							}
						}
						println!("Length of `{}` ", file_path);
						images.push(img_name.clone());
						file_path = file_path.replace(&img_name, "");
					}
				
			}
			let r = images.clone();
			let c = r.len();
			for (i, x) in r.iter().enumerate() {
				images[i] = format!("'{}'", x);
			}
			let images = format!("ARRAY[{}]", images.join(","));
			if category == "car_hire" {
				if c > 0 {
					let q = format!("UPDATE car SET car_images={} WHERE car_id='{}'", images, car_id);
					db.execute(q.as_str(), &[]).await.unwrap();
				} else {
					println!("{}", images);
					let token = 10.0;
					let daily_price: f64 = daily_price.parse().unwrap();
					let daily_down_payment: f64 = daily_down_payment.parse().unwrap();
					let q = format!(
						"INSERT INTO car (car_id,car_images, model, owner_id, location, description, daily_amount, daily_downpayment_amt, available,booking_tokens)
				VALUES
				  ('{}','{}', {}, '{}', '{}', '{}', {}, {}, {},{})",
						car_id, images, model, user_id, location, description, daily_price, daily_down_payment, available, token
					);
					db.execute(q.as_str(), &[]).await.unwrap();
				}
			} else if category == "taxi" {
				if c > 0 {
					let q = format!("UPDATE taxi SET image_paths={} WHERE taxi_id='{}'", images, car_id);
					db.execute(q.as_str(), &[]).await.unwrap();
				}
				for column in &columns {
					let query = format!("UPDATE taxi_verifications SET {}=$1 WHERE driver_id=$2", column);
					match db.0.execute(&query, &[&"Pending", &user_id]).await {
						Ok(_) => return StatusCode::OK,
						_ => return StatusCode::INTERNAL_SERVER_ERROR,
					}
				}
			}
			return 	StatusCode::OK
			},
			Ok(None) => break StatusCode::OK,
			Err(err) => {
				eprintln!("Error processing multipart upload: {}", err);
				break StatusCode::INTERNAL_SERVER_ERROR;
			}
		}
	}

}

fn save_file(parent_dir_path: &str, filename: &str, data: &[u8]) {
	let path = format!("{}{}", parent_dir_path, filename);
	let mut file = fs::File::create(path).unwrap();
	file.write_all(data).unwrap();
}

pub async fn delete_car(db: State<DbClient>, car_details: Json<Value>) -> StatusCode {
	let car_id = car_details["car_id"].as_str().unwrap();
	let owner_id = car_details["owner_id"].as_str().unwrap();
	let q = format!("DELETE FROM car WHERE car_id='{}' AND owner_id='{}'", car_id, owner_id);
	let x = db.execute(q.as_str(), &[]).await.unwrap();
	if x == 0 {
		return StatusCode::NOT_FOUND;
	}
	StatusCode::OK
}
