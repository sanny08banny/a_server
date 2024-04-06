use axum::Json;
use hyper::StatusCode;
use serde_json::{json, Value};

use crate::db_client;

#[derive(serde::Deserialize, serde::Serialize, Debug)]
pub struct User {
	email: String,
	password: String,
	notification_id: String,
}

pub async fn create_user(user: Json<User>) ->StatusCode{
	let g = db_client().await;
	let user = user.0;
	println!("{:?}", user);
	let email = user.email;
	let password = user.password;
	let notification_id = user.notification_id;
	let r_tokens = 600.00;
	let is_admin = false;
	let is_driver = false;
	let q = format!(
		"INSERT INTO users (email,password,tokens,isadmin,isdriver,notification_token) VALUES ('{}','{}','{}','{}','{}','{}')",
		email, password, r_tokens, is_admin, is_driver, notification_id
	);
	let res=g.execute(q.as_str(), &[]).await;
	if res.is_err(){
		return StatusCode::NOT_MODIFIED;
	}
	StatusCode::OK
}

pub async fn user_login(user: Json<User>) -> Result<Json<Value>, StatusCode> {
	let g = db_client().await;
	let user = user.0;
	let email = user.email;
	let password = user.password;
	let q = format!("SELECT * FROM users WHERE email='{}' AND password='{}'", email, password);
	let rows = g.query(q.as_str(), &[]).await.unwrap();
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

pub async fn change_category(j: Json<Value>) -> Json<Value> {
	let g = db_client().await;
	println!("{}", j.0);
	let j = j.0;
	let id: u32 = j["user_id"].as_str().unwrap().parse().unwrap();
	let category = j["category"].as_str().unwrap();
	if category == "driver" {
		let q = format!("UPDATE users SET isdriver=true WHERE user_id='{}'", id);
		let query=g.execute(q.as_str(), &[]).await;
		if query.is_err(){
			let p = json!({"user_id":id,"is_driver":false});
			return Json(p);
		}
		let p = json!({"user_id":id,"is_driver":true});
		return Json(p);
	} else if category == "admin" {
		let q = format!("UPDATE users SET isadmin=true WHERE user_id='{}'", id);
		let query=g.execute(q.as_str(), &[]).await;
		if query.is_err(){
			let p = json!({"user_id":id,"is_admin":false});
			return Json(p);
		}
		let p = json!({"user_id":id,"is_admin":true});
		return Json(p);
	} else if category == "normal" {
		let q = format!("UPDATE users SET isadmin=false,isdriver=false WHERE user_id='{}'", id);
		let query=g.execute(q.as_str(), &[]).await;
		if query.is_err(){
			let p = json!({"user_id":id,"is_admin":false,"is_driver":false});
			return Json(p);
		}
		let p = json!({"user_id":id,"is_admin":false,"is_driver":false});
		return Json(p);
	}
	let p = json!({"user_id":id,"is_admin":false,"is_driver":false});
	return Json(p);
}

pub async fn delete_user(j: Json<Value>)->StatusCode{
	let g = db_client().await;
	let j = j.0;
	let email= j["email"].as_str().unwrap();
	let pwd = j["password"].as_str().unwrap();
	let q = format!("DELETE FROM users WHERE email='{}' AND password='{}'",email,pwd);
	let y=g.execute(q.as_str(), &[]).await;
	if y.is_err(){
		return StatusCode::NOT_MODIFIED;
	}
	StatusCode::OK
}