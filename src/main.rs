use axum::{
    body::{Body, Bytes},
    extract::{Json, Multipart, Path},
    routing::{get, post, Router},
};
use hyper::Response;
use serde_json::Value;
use std::{
    fs::{self, File},
    io::{BufReader, Read},
    net::SocketAddr,
};
use tower_http::cors::CorsLayer;
mod payment_gateway;
mod r_tokens;
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
        .route("/houses", get(handler))
        .route("/houses/:path", get(image_handler))
        .route("/houses/download/:path", get(download_img))
        .route("/buyr", post(process_payment))
        .route("/houses/upload/:filename", post(img_upload))
        .route("/houses/mult_upload", post(mult_upload))
        .route("/path", post(call_back_url))
        .layer(CorsLayer::permissive());
    axum::Server::bind(&addr.trim().parse().expect("Invalid address"))
        .serve(app.into_make_service_with_connect_info::<SocketAddr>())
        .await
        .unwrap();
}
async fn handler() -> Json<Value> {
    let y = fs::read_to_string("test.json").expect("json file not found");
    let x: Value = serde_json::from_str(&y).expect("invalid json");
    Json(x)
}
async fn call_back_url(j: Json<Value>) {
    println!("Saf says:: {}", j.0);
}
async fn image_handler(path: Path<String>) -> Response<Body> {
    println!("file path: {:?}", path.0);
    let h = File::open(path.0).expect("file not found");
    let mut buf_reader = BufReader::new(h);
    let mut contents = Vec::new();
    buf_reader.read_to_end(&mut contents).unwrap();
    Response::new(Body::from(contents))
}

async fn download_img(path: Path<String>) -> Response<Body> {
    let h = File::open(path.0.clone()).expect("file not found");
    let mut buf_reader = BufReader::new(h);
    let mut contents = Vec::new();
    buf_reader.read_to_end(&mut contents).unwrap();
    let attachment = format!("attachment; filename={}", path.0);
    Response::builder()
        .header("Content-Disposition", attachment)
        .body(Body::from(contents))
        .unwrap()
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

async fn mult_upload(mut multipart: Multipart) {
    while let Some(field) = multipart.next_field().await.unwrap() {
        let name = field.name().unwrap().to_string();
        let data = field.bytes().await.unwrap();
        println!("Length of `{}` is {} bytes", name, data.len());
    }
}
// Stats page
// Customer support AI
// email server smtp
