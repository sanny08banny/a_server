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
    let r_tokens = 600.00;
    let is_admin = false;
    let is_driver = false;
    let q = format!(
        "INSERT INTO users (email,password,tokens,isadmin,isdriver) VALUES ('{}','{}','{}','{}','{}')",
        email, password, r_tokens, is_admin, is_driver
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

pub async fn change_category(j: Json<Value>)->Json<Value>{
 let g=db_client().await;
   println!("{}",j.0);
    let j=j.0;
    let id:u32=j["user_id"].as_str().unwrap().parse().unwrap();
    let category=j["category"].as_str().unwrap();
    if category=="driver"{
        let q=format!("UPDATE users SET isdriver=true WHERE user_id='{}'",id);
        g.execute(q.as_str(),&[]).await.unwrap();
        let p=json!({"user_id":id,"is_driver":true});
        return Json(p);
    }else if category=="admin" {
        let q=format!("UPDATE users SET isadmin=true WHERE user_id='{}'",id);
        g.execute(q.as_str(),&[]).await.unwrap();
        let p=json!({"user_id":id,"is_admin":true});
        return Json(p);
    }else if category=="normal"{
        let q=format!("UPDATE users SET isadmin=false,isdriver=false WHERE user_id='{}'",id);
        g.execute(q.as_str(),&[]).await.unwrap();
        let p=json!({"user_id":id,"is_admin":false,"is_driver":false});
        return Json(p);
    }
    let p=json!({"user_id":id,"is_admin":false,"is_driver":false});
    return Json(p);
}
