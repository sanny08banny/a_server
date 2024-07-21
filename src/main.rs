use crate::db_client::db_client;
use axum::routing::{get, post, Router};
use cars::{
	cars::{accept_book, get_cars, multi_upload, Car},
	taxi::{get_unverified_document, get_unverified_documents, get_unverified_taxis, verify_document},
};
use fcm_t::{fcm::req_ride, token::update_token};
use file_server::file_handler;
use payment_gateway::mpesa_payment_gateway::{call_back_url, process_payment};
use review::{get_review, create_review};
use tokens::r_tokens::query_token;
use tower_http::cors::CorsLayer;
use users::{change_category, create_user, user_login};

mod cars;
mod db_client;
mod ecryption_engine;
mod fcm_t;
mod file_server;
mod payment_gateway;
mod review;
mod search;
mod tokens;
mod users;

use crate::search::search;

#[tokio::main]
async fn main() {
	let listener = tokio::net::TcpListener::bind("0.0.0.0:4000").await.unwrap();
	let db = db_client().await;

	let app = Router::new()
		.route("/v1/cars", get(get_cars))
		.route("/v1/car/:parent_folder/:category/:owner_id/:car_id/:file_name", get(file_handler))
		.route("/v1/accept_book", post(accept_book))
		.route("/v1/process_payment", post(process_payment))
		.route("/v1/car/multi_upload", post(multi_upload))
		.route("/v1/user/tokens", post(query_token))
		.route("/v1/path", post(call_back_url))
		.route("/v1/user/new", post(create_user))
		.route("/v1/user/login", post(user_login))
		.route("/v1/car/review", post(get_review))
		.route("/v1/car/create_review", post(post(create_review)))
		.route("/v1/user/admin_req", post(change_category))
		.route("/v1/req_ride", post(req_ride))
		.route("/v1/book_car", post(fcm_t::fcm::book_car))
		.route("/v1/token_update/:user_id/:token", get(update_token))
		.route("/v1/driver_response", post(fcm_t::token::driver_response))
		.route("/v1/search", post(search))
		.route("/v1/delete_user", post(users::delete_user))
		.route("/v1/delete_car", post(cars::cars::delete_car))
		.route("/v1/init_taxi", post(cars::taxi::init_taxi))
		.route("/v1/taxi_details", post(cars::taxi::taxi_details))
		.route("/v1/ride_req_status", post(fcm_t::fcm::ride_request_status))
		.route("/v1/book_req_status", post(fcm_t::fcm::book_request_status))
		// taxi verification
		.route("/v1/get_unverified_taxis", get(get_unverified_taxis))
		.route("/v1/get_unverified_documents", post(get_unverified_documents))
		.route("/v1/get_unverified_document", post(get_unverified_document))
		.route("/v1/verify_document", post(verify_document))
		.with_state(db)
		.layer(CorsLayer::permissive());

	axum::serve(listener, app).await.unwrap()
}
