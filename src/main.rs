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
        .route("/car/book", post(book))
        .route("/car/:owner_id/:car_id/:file_name", get(download_img))
        .route("/buyr", post(process_payment))
        .route("/car/upload/:filename", post(img_upload))
        .route("/car/mult_upload", post(mult_upload))
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
    let q = format!(
        "INSERT INTO users (email,password) VALUES ('{}','{}')",
        email, password
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
        let available: bool = row.get::<_,bool>("available");
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
    // let y = fs::read_to_string("src/dummy/cars.json").expect("json file not found");
    // let x: Value = serde_json::from_str(&y).expect("invalid json");
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

async fn book(req_details: Json<BookingDetails>) {
    let det = req_details.0;
}

async fn mult_upload(mut multipart: Multipart) {
    let mut admin_id = String::new();
    let mut car_id = String::new();
    let mut img_path = String::new();
    let mut index = 0;
    let g = db_client().await;
    while let Some(field) = multipart.next_field().await.unwrap() {
        let name = field.name().unwrap().to_string();
        match name.as_str() {
            "admin_id" => {
                admin_id = field.text().await.unwrap();
            }
            "car_id" => {
                let q = format!(
                    "INSERT INTO car (owner_id,car_id) VALUES ('{}','{}')",
                    admin_id, car_id
                );
                g.execute(q.as_str(), &[]).await.unwrap();
                car_id = field.text().await.unwrap();
                img_path = format!("images/{}/{}/", admin_id, car_id);
                match fs::create_dir_all(&img_path) {
                    Ok(_) => {}
                    Err(e) => {
                        println!("failed to create directories {}", e)
                    }
                }
            }
            "model" => {
                let q = format!(
                    "UPDATE car SET model = '{}' WHERE owner_id='{}' AND car_id='{}'",
                    field.text().await.unwrap(),
                    admin_id,
                    car_id
                );
                g.execute(q.as_str(), &[]).await.unwrap();
            }
            "location" => {
                let q = format!(
                    "UPDATE car SET location = '{}' WHERE owner_id='{}' AND car_id='{}'",
                    field.text().await.unwrap(),
                    admin_id,
                    car_id
                );
                g.execute(q.as_str(), &[]).await.unwrap();
            }
            "description" => {
                let q = format!(
                    "UPDATE car SET description = '{}' WHERE owner_id='{}' AND car_id='{}'",
                    field.text().await.unwrap(),
                    admin_id,
                    car_id
                );
                g.execute(q.as_str(), &[]).await.unwrap();
            }
            "daily_price" => {
                let q = format!(
                    "UPDATE car SET daily_amount = '{}' WHERE owner_id='{}' AND car_id='{}'",
                    field.text().await.unwrap(),
                    admin_id,
                    car_id
                );
                g.execute(q.as_str(), &[]).await.unwrap();
            }
            "hourly_price" => {
                let q = format!(
                    "UPDATE car SET hourly_amount = '{}' WHERE owner_id='{}' AND car_id='{}'",
                    field.text().await.unwrap(),
                    admin_id,
                    car_id
                );
                g.execute(q.as_str(), &[]).await.unwrap();
            }
            "daily_down_payment" => {
                let q=format!("UPDATE car SET daily_downpayment_amt = '{}' WHERE owner_id='{}' AND car_id='{}'",field.text().await.unwrap(),admin_id,car_id);
                g.execute(q.as_str(), &[]).await.unwrap();
            }
            "hourly_down_payment" => {
                let q=format!("UPDATE car SET hourly_downpayment_amt = '{}' WHERE owner_id='{}' AND car_id='{}'",field.text().await.unwrap(),admin_id,car_id);
                g.execute(q.as_str(), &[]).await.unwrap();
            }
            _ => {
                let mut img_file_format = match field.content_type() {
                    Some(x) => x,
                    None => "image/png",
                }
                .to_owned();
                // remove image/ from the content type
                img_file_format = img_file_format.replace("image/", "");
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
                let q=format!("UPDATE car SET car_images = array_append(car_images, '{}') WHERE owner_id='{}' AND car_id='{}'",img_name,admin_id,car_id);
                g.execute(q.as_str(), &[]).await.unwrap();
                println!("Length of `{}` ", img_path);
                img_path = img_path.replace(&img_name, "");
            }
        }
    }
}
