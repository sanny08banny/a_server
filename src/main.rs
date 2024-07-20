use crate::db_client::db_client;
use axum::routing::{get, post, Router};
use cars::cars::{accept_book, get_cars, mult_upload, Car};
use fcm_t::{fcm::req_ride, token::update_token};
use file_server::file_handler;
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
mod file_server;
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
		.route("/v1/cars", get(get_cars))
		.route("/v1/car/:parent_folder/:category/:owner_id/:car_id/:file_name", get(file_handler))
		.route("/v1/accept_book", post(accept_book))
		.route("/v1/buyr", post(process_payment))
		.route("/v1/car/mult_upload", post(mult_upload))
		.route("/v1/user/tokens", post(query_token))
		.route("/v1/path", post(call_back_url))
		.route("/v1/user/new", post(create_user))
		.route("/v1/user/login", post(user_login))
		.route("/v1/car/review", post(car_review))
		.route("/v1/car/create_review", post(post(post_review)))
		.route("/v1/user/admin_req", post(change_category))
		.route("/v1/req_ride", post(req_ride))
		.route("/v1/book_car", post(fcm_t::fcm::book_car))
		.route("/v1/token_update/:user_id/:token", get(update_token))
		.route("/v1/driver_response", post(fcm_t::token::driver_response))
		.route("/v1/search", post(search))
		.route("/v1/delete_user", post(users::users::delete_user))
		.route("/v1/delete_car", post(cars::cars::delete_car))
		.route("/v1/init_taxi", post(cars::taxi::init_taxi))
		.route("/v1/taxi_details", post(cars::taxi::taxi_details))
		.route("/v1/ride_req_status", post(fcm_t::fcm::ride_req_status))
		.route("/v1/book_req_status", post(fcm_t::fcm::book_req_status))
		.layer(CorsLayer::permissive());
	axum::Server::bind(&addr.trim().parse().expect("Invalid address"))
		.serve(app.into_make_service_with_connect_info::<SocketAddr>())
		.await
		.unwrap();
}
