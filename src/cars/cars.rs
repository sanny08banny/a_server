use std::fs;

use axum::{extract::Multipart, Json};
use hyper::StatusCode;

use crate::db_client;

#[derive(serde::Deserialize, serde::Serialize)]
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

#[derive(serde::Deserialize)]
pub struct BookingDetails {
	user_id: String,
	car_id: String,
	description: String,
}

pub async fn handler() -> Json<Vec<Car>> {
	let g = db_client().await;
	let q = "SELECT * FROM car";
	let rows = g.query(q, &[]).await.unwrap();
	let mut x = Vec::new();
	for row in rows {
		let owner_id: String = row.get::<_, String>("owner_id");
		let car_id: String = row.get::<_, String>("car_id");
		let model: String = row.get::<_, String>("model");
		let location: String = row.get::<_, String>("location");
		let description: String = row.get::<_, String>("description");
		let daily_amount: f64 = row.get::<_, f64>("daily_amount");
		let daily_downpayment_amt: f64 = row.get::<_, f64>("daily_downpayment_amt");
		let car_images: Vec<String> = row.get::<_, Vec<String>>("car_images");
		let available: bool = row.get::<_, bool>("available");
		let car = Car {
			car_images,
			model,
			car_id,
			owner_id,
			location,
			description,
			amount: daily_amount,
			downpayment_amt: daily_downpayment_amt,
			available,
		};
		x.push(car);
	}
	Json(x)
}

pub async fn accept_book(req_details: Json<BookingDetails>) -> StatusCode {
	let det = req_details.0;
	if det.description == "book" {
		let g = db_client().await;
		let y = format!("SELECT booking_tokens FROM car WHERE car_id='{}'", det.car_id);
		let rows = g.query(y.as_str(), &[]).await.unwrap();
		let mut booking_tokens = 0.00;
		for row in rows {
			booking_tokens = row.get::<_, f64>("booking_tokens");
		}
		let x = format!("SELECT tokens FROM users WHERE user_id='{}'", det.user_id);
		let rows = g.query(x.as_str(), &[]).await.unwrap();
		let mut user_tokens = 0.00;
		for row in rows {
			user_tokens = row.get::<_, f64>("tokens");
		}
		if user_tokens < booking_tokens {
			return StatusCode::EXPECTATION_FAILED;
		}
		let new_user_tokens = user_tokens - booking_tokens;
		let x = format!("UPDATE users SET tokens='{}' WHERE user_id='{}'", new_user_tokens, det.user_id);
		g.execute(x.as_str(), &[]).await.unwrap();
		return StatusCode::OK;
	} else if det.description == "unbook" {
		return StatusCode::OK;
	}
	return StatusCode::NOT_FOUND;
}

pub async fn mult_upload(mut multipart: Multipart) {
	let mut admin_id = String::new();
	let mut car_id = String::new();
	let mut model = String::new();
	let mut location = String::new();
	let mut description = String::new();
	let mut daily_price = String::new();
	let mut daily_down_payment = String::new();
	let mut available = true;
	let mut images: Vec<String> = Vec::new();
	let mut img_path = String::new();
	let mut index = 0;
	let g = db_client().await;
	while let Some(field) = multipart.next_field().await.unwrap() {
		let name = field.name().unwrap().to_string();
		println!("{:?}", name);
		match name.as_str() {
			"admin_id" => {
				admin_id = field.text().await.unwrap().replace("\"", "");
			}
			"car_id" => {
				car_id = field.text().await.unwrap().replace("\"", "");
				img_path = format!("images/{}/{}/", admin_id, car_id);
				match fs::create_dir_all(&img_path) {
					Ok(_) => {}
					Err(e) => {
						println!("failed to create directories {}", e)
					}
				}
			}
			"model" => {
				model = field.text().await.unwrap().replace("\"", "");
			}
			"location" => {
				location = field.text().await.unwrap().replace("\"", "");
			}
			"description" => {
				description = field.text().await.unwrap().replace("\"", "");
			}
			"daily_price" => {
				daily_price = field.text().await.unwrap().replace("\"", "");
			}
			"daily_down_payment" => {
				daily_down_payment = field.text().await.unwrap().replace("\"", "");
			}
			"available" => {
				available = field.text().await.unwrap().parse().unwrap();
			}
			_ => {
				let mut img_file_format = match field.content_type() {
					Some(x) => x,
					None => "image/png",
				}
				.to_owned();
				// remove image/ from the content type
				img_file_format = img_file_format.replace("image/", "");
				img_file_format = img_file_format.replace("*", "jpeg");
				let img_name = format!("img_{}.{}", index, img_file_format);
				println!("file format {}", img_file_format);
				let img = image::load_from_memory(&field.bytes().await.unwrap()).unwrap();
				img_path.push_str(&img_name);
				match img.save(&img_path) {
					Ok(_) => {
						index += 1;
					}
					Err(e) => {
						println!("Failed to save image: {}", e)
					}
				}
				println!("Length of `{}` ", img_path);
				images.push(img_name.clone());
				img_path = img_path.replace(&img_name, "");
			}
		}
	}
	let r = images.clone();
	for (i, x) in r.iter().enumerate() {
		images[i] = format!("'{}'", x);
	}
	let images = format!("ARRAY[{}]", images.join(","));
	println!("{}", images);
	let token = 10.0;
	let daily_price: f64 = daily_price.parse().unwrap();
	let daily_down_payment: f64 = daily_down_payment.parse().unwrap();
	let q = format!(
		"INSERT INTO car (car_id, car_images, model, owner_id, location, description, daily_amount, daily_downpayment_amt, available,booking_tokens)
        VALUES
          ('{}', {}, '{}', '{}', '{}', '{}', {}, {}, {},{})",
		car_id, images, model, admin_id, location, description, daily_price, daily_down_payment, available, token
	);
	g.execute(q.as_str(), &[]).await.unwrap();
}
