use axum::{extract::State, Json};
use base64::Engine;
use chrono::Local;
use hyper::StatusCode;
use serde_json::{json, Value};

use crate::{db_client::DbClient, ecryption_engine::CUSTOM_ENGINE};

#[derive(serde::Deserialize, serde::Serialize, Debug)]
pub enum UserType {
	Driver,
	Owner,
	Rider,
	Admin,
}

#[derive(serde::Deserialize, serde::Serialize, Debug)]
pub struct User {
	user_type: UserType,
	email: String,
	password: String,
	name: String,
	notification_id: String,
}

pub async fn create_user(db: State<DbClient>, user: Json<User>) -> StatusCode {
	let user = user.0;

	let is_driver = matches!(user.user_type, UserType::Driver);
	let is_admin = matches!(user.user_type, UserType::Admin);

	let timestamp = Local::now().format("%Y%m%d%H%M%S").to_string();
	let input = format!("{}-{}", &user.email, timestamp);
	let user_id = CUSTOM_ENGINE.encode(input);

	let statement = "INSERT INTO users (user_id,email,password,tokens,isadmin,isdriver,notification_token,user_name) VALUES ($1,$2,$3,$4,$5,$6,$7,$8)";
	let res = db
		.execute(statement, &[&user_id, &user.email, &user.password, &600i64, &is_admin, &is_driver, &user.notification_id, &user.name])
		.await;

	match res {
		Ok(_) => StatusCode::OK,
		Err(_) => StatusCode::NOT_MODIFIED,
	}
}

pub async fn user_login(db: State<DbClient>, user: Json<User>) -> Result<Json<Value>, StatusCode> {
	let user = user.0;
	let email = user.email;
	let password = user.password;
	let q = format!("SELECT * FROM users WHERE email='{}' AND password='{}'", email, password);
	let rows = db.query(q.as_str(), &[]).await.unwrap();
	let mut x = String::new();
	let mut is_admin = false;
	let mut is_driver = false;
	for row in rows {
		let id: i32 = row.get(0);
		is_admin = row.get("isadmin");
		is_driver = row.get("isdriver");
		x = id.to_string();
	}
	if x.is_empty() {
		return Err(StatusCode::UNAUTHORIZED);
	}
	let x = json!({"user_id":x,"is_admin":is_admin,"is_driver":is_driver});
	Ok(Json(x))
}

pub async fn change_category(db: State<DbClient>, j: Json<Value>) -> Json<Value> {
	println!("{}", j.0);
	let j = j.0;
	let id: u32 = j["user_id"].as_str().unwrap().parse().unwrap();
	let category = j["category"].as_str().unwrap();
	if category == "driver" {
		let q = format!("UPDATE users SET isdriver=true WHERE user_id='{}'", id);
		let query = db.execute(q.as_str(), &[]).await;
		if query.is_err() {
			let p = json!({"user_id":id,"is_driver":false});
			return Json(p);
		}
		let p = json!({"user_id":id,"is_driver":true});
		return Json(p);
	} else if category == "admin" {
		let q = format!("UPDATE users SET isadmin=true WHERE user_id='{}'", id);
		let query = db.execute(q.as_str(), &[]).await;
		if query.is_err() {
			let p = json!({"user_id":id,"is_admin":false});
			return Json(p);
		}
		let p = json!({"user_id":id,"is_admin":true});
		return Json(p);
	} else if category == "normal" {
		let q = format!("UPDATE users SET isadmin=false,isdriver=false WHERE user_id='{}'", id);
		let query = db.execute(q.as_str(), &[]).await;
		if query.is_err() {
			let p = json!({"user_id":id,"is_admin":false,"is_driver":false});
			return Json(p);
		}
		let p = json!({"user_id":id,"is_admin":false,"is_driver":false});
		return Json(p);
	}
	let p = json!({"user_id":id,"is_admin":false,"is_driver":false});
	Json(p)
}

pub async fn delete_user(db: State<DbClient>, j: Json<Value>) -> StatusCode {
	let j = j.0;
	let email = j["email"].as_str().unwrap();
	let pwd = j["password"].as_str().unwrap();
	let q = format!("DELETE FROM users WHERE email='{}' AND password='{}'", email, pwd);
	let y = db.execute(q.as_str(), &[]).await;
	if y.is_err() {
		return StatusCode::NOT_MODIFIED;
	}
	StatusCode::OK
}
