use axum::routing::{get, post, Router};
use cars::cars::{Car, handler, mult_upload, book};
use image_server::image_handler;
use payment_gateway::mpesa_payment_gateway::{process_payment, call_back_url};
use review::review::{car_review, post_review};
use tokens::r_tokens::query_token;
use users::users::{create_user, user_login, change_category};
use std::net::SocketAddr;
use crate::db_client::db_client;
use tower_http::cors::CorsLayer;
mod ecryption_engine;
mod image_server;
mod payment_gateway;
mod tokens;
mod verification;
mod review;
mod search;
mod location;
mod db_client;
mod users;
mod cars;
use crate::search::search::search;

#[tokio::main]
async fn main() {
    let addr = "0.0.0.0:4000";
    let app = Router::new()
        .route("/cars", get(handler))
        .route("/car_img", get(image_handler))
        .route("/car/action", post(book))
        .route("/buyr", post(process_payment))
        .route("/car/mult_upload", post(mult_upload))
        .route("/user/tokens", post(query_token))
        .route("/path", post(call_back_url))
        .route("/user/new", post(create_user))
        .route("/user/login", post(user_login))
        .route("/car/review", post(car_review))
        .route("/car/create_review", post(post(post_review)))
        .route("/user/admin_req", post(change_category))
        .route("/search", post(search))
        .layer(CorsLayer::permissive());
    axum::Server::bind(&addr.trim().parse().expect("Invalid address"))
        .serve(app.into_make_service_with_connect_info::<SocketAddr>())
        .await
        .unwrap();
}

