use axum::{
    body::{Body, Bytes},
    extract::{Json, Multipart, Path},
    response::Response,
    routing::{get, post, Router},
};
use hyper::StatusCode;
use image_server::image_handler;
use serde_json::{json, Value};
use std::{
    fs::{self, File},
    io::{BufReader, Read},
    net::SocketAddr,
};
use tokio_postgres::{Client, NoTls};
use tower_http::cors::CorsLayer;
mod ecryption_engine;
mod image_server;
mod payment_gateway;
mod r;
mod rental;
mod verification;
use payment_gateway::mpesa_payment_gateway::MpesaPaymentProcessor;

#[derive(serde::Deserialize)]
struct PaymentDetails {
    amount: f32,
    phone_number: String,
    description: String,
}
#[derive(serde::Deserialize)]
struct BookingDetails {
    user_id: String,
    car_id: String,
    description: String,
}
#[derive(serde::Deserialize, serde::Serialize)]
struct User {
    email: String,
    password: String,
}
#[derive(serde::Deserialize, serde::Serialize)]
struct Car {
    car_images: Vec<String>,
    model: String,
    car_id: String,
    owner_id: String,
    location: String,
    description: String,
    pricing: Pricing,
    available: bool,
}
#[derive(serde::Deserialize, serde::Serialize)]
struct Pricing {
    hourly: Amount,
    daily: Amount,
}
#[derive(serde::Deserialize, serde::Serialize)]
struct Amount {
    amount: f64,
    downpayment_amt: f64,
}
#[tokio::main]
async fn main() {
    let addr = "0.0.0.0:4000";
    let app = Router::new()
        .route("/cars", get(handler))
        .route("/car_img", get(image_handler))
        .route("/car/action", post(book))
        .route("/car/:owner_id/:car_id/:file_name", get(download_img))
        .route("/buyr", post(process_payment))
        .route("/car/upload/:filename", post(img_upload))
        .route("/car/mult_upload", post(mult_upload))
        .route("/user/tokens", post(query_token))
        .route("/path", post(call_back_url))
        .route("/user/new", post(create_user))
        .route("/user/login", post(user_login))
        .layer(CorsLayer::permissive());
    axum::Server::bind(&addr.trim().parse().expect("Invalid address"))
        .serve(app.into_make_service_with_connect_info::<SocketAddr>())
        .await
        .unwrap();
}

async fn db_client() -> Client {
    let host = "localhost";
    let user = "ubuntu";
    let password = "new_password";
    let dbname = "ubuntu";
    let config_string = format!(
        "host={} user={} password={} dbname={}",
        host, user, password, dbname
    );
    let (client, monitor) = tokio_postgres::connect(config_string.as_str(), NoTls)
        .await
        .unwrap();

    tokio::spawn(async move {
        if let Err(e) = monitor.await {
            eprintln!("Connection error: {}", e);
        }
    });

    client
}

async fn create_user(user: Json<User>) {
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
async fn user_login(user: Json<User>) -> Result<Json<Value>, StatusCode> {
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
    for row in rows {
        let id: i32 = row.get(0);
        x = id.to_string();
    }
    if x.is_empty() {
        return Err(StatusCode::UNAUTHORIZED);
    }
    Ok(Json(serde_json::to_value(x).unwrap()))
}

async fn handler() -> Json<Vec<Car>> {
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
        let hourly_amount: f64 = row.get::<_, f64>("hourly_amount");
        let daily_downpayment_amt: f64 = row.get::<_, f64>("daily_downpayment_amt");
        let hourly_downpayment_amt: f64 = row.get::<_, f64>("hourly_downpayment_amt");
        let car_images: Vec<String> = row.get::<_, Vec<String>>("car_images");
        let available: bool = row.get::<_, bool>("available");
        let car = Car {
            car_images,
            model,
            car_id,
            owner_id,
            location,
            description,
            pricing: Pricing {
                hourly: Amount {
                    amount: hourly_amount,
                    downpayment_amt: hourly_downpayment_amt,
                },
                daily: Amount {
                    amount: daily_amount,
                    downpayment_amt: daily_downpayment_amt,
                },
            },
            available,
        };
        x.push(car);
    }
    Json(x)
}

async fn call_back_url(j: Json<Value>) {
    println!("Saf says:: {}", j.0);
}
async fn download_img(params: Path<Vec<String>>) -> Response<Body> {
    let mut o = params.0.iter();
    let owner_id = o.next().unwrap();
    let car_id = o.next().unwrap();
    let file_name = o.next().unwrap();
    let p = format!("images/{}/{}/{}", owner_id, car_id, file_name);
    println!("{}", p);
    let h = File::open(p.clone()).expect("file not found");
    let mut buf_reader = BufReader::new(h);
    let mut contents = Vec::new();
    buf_reader.read_to_end(&mut contents).unwrap();
    Response::new(Body::from(contents))
}

async fn img_upload(path: Path<String>, body: Bytes) {
    let img = image::load_from_memory(&body).unwrap();
    img.save(path.0).unwrap();
}

async fn process_payment(payment_details: Json<PaymentDetails>) {
    let details = payment_details.0;
    let processor = MpesaPaymentProcessor::new(
        details.amount,
        details.phone_number.as_str(),
        details.description.as_str(),
    );
    println!("{:?}", processor.handle_payment().await);
}

async fn book(req_details: Json<BookingDetails>) -> StatusCode {
    let det = req_details.0;
    let g = db_client().await;
    let y = format!(
        "SELECT booking_tokens FROM car WHERE car_id='{}'",
        det.car_id
    );
    let rows = g.query(y.as_str(), &[]).await.unwrap();
    let mut booking_tokens = String::new();
    for row in rows {
        booking_tokens = row.get::<_, String>("booking_tokens");
    }
    let booking_tokens: f64 = booking_tokens.parse().unwrap();
    let x = format!("SELECT tokens FROM users WHERE user_id='{}'", det.user_id);
    let rows = g.query(x.as_str(), &[]).await.unwrap();
    let mut user_tokens = String::new();
    for row in rows {
        user_tokens = row.get::<_, String>("tokens");
    }
    let user_tokens: f64 = user_tokens.parse().unwrap();
    if user_tokens < booking_tokens {
        return StatusCode::EXPECTATION_FAILED;
    }
    let new_user_tokens = user_tokens - booking_tokens;
    let x = format!(
        "UPDATE users SET tokens='{}' WHERE user_id='{}'",
        new_user_tokens, det.user_id
    );
    g.execute(x.as_str(), &[]).await.unwrap();
    return StatusCode::OK;
}
async fn query_token(uid: Json<String>) -> String {
    let g = db_client().await;
    let uid = uid.0;
    let x = format!("SELECT tokens FROM users WHERE user_id='{}'", uid);
    let rows = g.query(x.as_str(), &[]).await.unwrap();
    let mut user_tokens = String::new();
    for row in rows {
        user_tokens = row.get::<_, String>("tokens");
    }
    user_tokens
}

async fn mult_upload(mut multipart: Multipart) {
    let mut admin_id = String::new();
    let mut car_id = String::new();
    let mut model = String::new();
    let mut location = String::new();
    let mut description = String::new();
    let mut daily_price = String::new();
    // let mut hourly_price = String::new();
    let mut daily_down_payment = String::new();
    // let mut hourly_down_payment = String::new();
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
                model = field.text().await.unwrap();
            }
            "location" => {
                location = field.text().await.unwrap();
            }
            "description" => {
                description = field.text().await.unwrap();
            }
            "daily_price" => {
                daily_price = field.text().await.unwrap();
            }
            "daily_down_payment" => {
                daily_down_payment = field.text().await.unwrap();
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
    let daily_price: f64 = daily_price.parse().unwrap();
    let daily_down_payment: f64 = daily_down_payment.parse().unwrap();
    let q = format!(
        "INSERT INTO car (car_id, car_images, model, owner_id, location, description, daily_amount, daily_downpayment_amt, available)
        VALUES
          ('{}', {}, '{}', '{}', {}, {}, {}, {}, {})",
          car_id,images,model,admin_id,location,description,daily_price,daily_down_payment,available
    );
    g.execute(q.as_str(), &[]).await.unwrap();
}
