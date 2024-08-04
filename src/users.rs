use axum::{extract::State, Json};
use base64::Engine;
use chrono::Local;
use hyper::StatusCode;
use serde_json::{json, Value};

use crate::{db_client::DbClient, encryption_engine::CUSTOM_ENGINE};

#[derive(serde::Deserialize, serde::Serialize, Debug)]
pub enum UserType {
	Driver,
	Owner,
	Rider,
	Admin,
}

#[derive(serde::Deserialize, serde::Serialize, Debug)]
pub struct User {
	email: String,
	password: String,
	name: String,
	tel:String,
	notification_id: String,
}

pub async fn create_user(db: State<DbClient>, user: Json<User>) -> StatusCode {
	let user = user.0;

	let timestamp = Local::now().format("%Y%m%d%H%M%S").to_string();
	let input = format!("{}-{}-{}", &user.email, timestamp,&user.tel);
	let user_id = CUSTOM_ENGINE.encode(input);

	let statement = "INSERT INTO users (user_id,email,user_phone,password,tokens,isadmin,isdriver,notification_token,user_name) VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9)";
	let res = db
		.execute(statement, &[&user_id, &user.email, &user.tel,&user.password, &600f64, &false, &false, &user.notification_id, &user.name])
		.await;
	println!("{:?}",res);
	match res {
		Ok(_) => StatusCode::OK,
		Err(_) => StatusCode::NOT_MODIFIED,
	}
}

#[derive(serde::Deserialize, serde::Serialize, Debug)]
pub struct Logins{
	email:String,
	password:String
}

pub async fn user_login(db: State<DbClient>, logins: Json<Logins>) -> Result<Json<Value>, StatusCode> {
	let logins = logins.0;
	let email = logins.email;
	let password = logins.password;
	let Some(row) = db.query_opt("SELECT user_id,isadmin,isdriver,user_name,user_phone FROM users WHERE email=$1 AND password=$2", &[&email,&password]).await.unwrap() else{
		return Err(StatusCode::UNAUTHORIZED);
	};
	let  user_id:&str =row.get(0) ;
	let  user_name:&str = row.get(3);
	let  user_phone:&str = row.get(4);
	Ok(Json(json!({"user_id":user_id,
		"user_name":user_name,"user_phone":user_phone})))
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
