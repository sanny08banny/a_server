use axum::Json;
use hyper::StatusCode;
use serde_json::{Value, json};

use crate::db_client;

#[derive(serde::Deserialize, serde::Serialize)]
pub struct User {
    email: String,
    password: String,
}


pub async fn create_user(user: Json<User>) {
    let g = db_client().await;
    let user = user.0;
    let email = user.email;
    let password = user.password;
    let r_tokens = 30.00;
    let q = format!(
        "INSERT INTO users (email,password,tokens) VALUES ('{}','{}','{}')",
        email, password, r_tokens
    );
    g.execute(q.as_str(), &[]).await.unwrap();
}

pub async fn user_login(user: Json<User>) -> Result<Json<Value>, StatusCode> {
    let g = db_client().await;
    let user = user.0;
    let email = user.email;
    let password = user.password;
    let q = format!(
        "SELECT * FROM users WHERE email='{}' AND password='{}'",
        email, password
    );
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
pub async fn change_category(j: Json<String>)->Json<Value>{
 let g=db_client().await;
   println!("{}",j.0);
    let j:u32=j.0.parse().unwrap();

    // update users set isadmin=true where user_id='1';
    let q=format!("UPDATE users SET isadmin=true WHERE user_id='{}'",j);
    g.execute(q.as_str(),&[]).await.unwrap();
    let p=json!({"user_id":j,"is_admin":true});
    Json(p)
}
