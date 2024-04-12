use crate::db_client::db_client;
use axum::routing::{get, post, Router};
use cars::cars::{accept_book, handler, mult_upload, Car};
use fcm_t::{fcm::req_ride, token::update_token};
use image_server::image_handler;
use payment_gateway::mpesa_payment_gateway::{call_back_url, process_payment};
use review::review::{car_review, post_review};
use std::net::SocketAddr;
use tokens::r_tokens::query_token;
use tower_http::cors::CorsLayer;
use users::users::{change_category, create_user, user_login};
mod cars;
mod db_client;
mod ecryption_engine;
mod fcm_t;
mod image_server;
mod location;
mod payment_gateway;
mod review;
mod search;
mod tokens;
mod users;
mod verification;
use crate::search::search::search;

#[tokio::main]
async fn main() {
	let addr = "0.0.0.0:4000";
	let app = Router::new()
		.route("/cars", get(handler))
		.route("/car/:owner_id/:car_id/:file_name", get(image_handler))
		.route("/accept_book", post(accept_book))
		.route("/buyr", post(process_payment))
		.route("/car/mult_upload", post(mult_upload))
		.route("/user/tokens", post(query_token))
		.route("/path", post(call_back_url))
		.route("/user/new", post(create_user))
		.route("/user/login", post(user_login))
		.route("/car/review", post(car_review))
		.route("/car/create_review", post(post(post_review)))
		.route("/user/admin_req", post(change_category))
		.route("/req_ride", post(req_ride))
		.route("/book_car", post(fcm_t::fcm::book_car))
		.route("/token_update/:user_id/:token", get(update_token))
		.route("/driver_response", post(fcm_t::token::driver_response))
		.route("/search", post(search))
		.route("/delete_user", post(users::users::delete_user))
		.layer(CorsLayer::permissive());
	axum::Server::bind(&addr.trim().parse().expect("Invalid address"))
		.serve(app.into_make_service_with_connect_info::<SocketAddr>())
		.await
		.unwrap();
}
