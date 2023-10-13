use axum::{
    body::{Body, Bytes},
    extract::{Json, Multipart, Path},
    response::Response,
    routing::{get, post, Router},
};
use image_server::image_handler;
use serde_json::Value;
use std::{
    fs::{self, File},
    io::{BufReader, Read},
    net::SocketAddr,
};
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
#[tokio::main]
async fn main() {
    let addr = "0.0.0.0:4000";
    let app = Router::new()
        .route("/cars", get(handler))
        .route("/car_img", get(image_handler))
        .route("/car/book", post(book))
        .route("/car/download/:owner_id/:car_id/:file_name", get(download_img))
        .route("/buyr", post(process_payment))
        .route("/car/upload/:filename", post(img_upload))
        .route("/car/mult_upload", post(mult_upload))
        .route("/path", post(call_back_url))
        .layer(CorsLayer::permissive());
    axum::Server::bind(&addr.trim().parse().expect("Invalid address"))
        .serve(app.into_make_service_with_connect_info::<SocketAddr>())
        .await
        .unwrap();
}
async fn handler() -> Json<Value> {
    let y = fs::read_to_string("src/dummy/cars.json").expect("json file not found");
    let x: Value = serde_json::from_str(&y).expect("invalid json");
    Json(x)
}
async fn call_back_url(j: Json<Value>) {
    println!("Saf says:: {}", j.0);
}
async fn download_img(owner_id: Path<String>,car_id:Path<String>,file_name:Path<String>) -> Response<Body> {
    let p= format!("images/{}/{}/{}/",owner_id.0,car_id.0,file_name.0);
    let h = File::open(p.clone()).expect("file not found");
    let mut buf_reader = BufReader::new(h);
    let mut contents = Vec::new();
    buf_reader.read_to_end(&mut contents).unwrap();
    // let attachment = format!("attachment; filename={}", p);
    // Response::builder()
    //     .header("Content-Disposition", attachment)
    //     .body(Body::from(contents))
    //     .unwrap()
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

async fn book(req_details: Json<Value>) {
    let details = req_details.0;
    // * {userid:"",car_id:"",pricing_option:""}
    // TODO: Check the user's and car's validation status
    // TODO: Respond na owner's contacts
}
async fn mult_upload(mut multipart: Multipart) {
    let mut admin_id = String::new();
    let mut car_id = String::new();
    let mut img_path = String::new();
    while let Some(field) = multipart.next_field().await.unwrap() {
        let name = field.name().unwrap().to_string();
        println!("type {:?}", field.content_type());
        if name == "admin_id" {
            admin_id = field.text().await.unwrap();
        } else if name == "car_id" {
            car_id = field.text().await.unwrap();
            img_path = format!("images/{}/{}/", admin_id, car_id);
            match fs::create_dir_all(&img_path) {
                Ok(_) => {}
                Err(e) => {
                    println!("failed to create directories {}", e)
                }
            }
        } else {
            let img = image::load_from_memory(&field.bytes().await.unwrap()).unwrap();
            
            img_path.push_str(&name);
            match img.save(&img_path) {
                Ok(_) => {}
                Err(e) => {
                    println!("Failed to save image: {}", e)
                }
            }
            println!("Length of `{}` ", img_path);
            img_path = img_path.replace(&name, "");
        }
    }
}
