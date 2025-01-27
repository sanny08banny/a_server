use axum::{
	extract::DefaultBodyLimit,
	routing::{get, post, Router},
};
use cars::{
	cars::{get_cars, handle_book, multi_upload, Car},
	taxi::{accept_ride_request, decline_ride_request, get_unverified_document, get_unverified_documents, get_unverified_taxis, reqest_ride, taxi_price, verify_document},
};
use fcm_t::token::update_token;
use file_server::{file_handler, get_car};
use payment_gateway::mpesa_payment_gateway::{call_back_url, process_payment};
use review::{create_review, get_review};
use tokens::r_tokens::query_token;
use tower_http::cors::CorsLayer;
use users::{change_category, create_user, user_login};

mod cars;
mod db_client;
mod encryption_engine;
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
	let db = db_client::db_client().await;
	env_logger::init();
	let app = Router::new()
		.route("/v1/cars", get(get_cars))
		.route("/v1/taxi/image/:driver_id/:file_name", get(file_handler))
		.route("/v1/car/image/:owner_id/:car_id/:file_name", get(get_car))
		.route("/v1/book", post(handle_book))
		.route("/v1/process_payment", post(process_payment))
		.route("/v1/car/multi_upload", post(multi_upload))
		.layer(DefaultBodyLimit::disable())
		.route("/v1/user/tokens", post(query_token))
		.route("/v1/path", post(call_back_url))
		.route("/v1/user/new", post(create_user))
		.route("/v1/user/login", post(user_login))
		.route("/v1/car/review", post(get_review))
		.route("/v1/car/create_review", post(post(create_review)))
		.route("/v1/user/admin_req", post(change_category))
		.route("/v1/taxi/request", post(reqest_ride))
		.route("/v1/token_update/:user_id/:token", get(update_token))
		.route("/v1/taxi/accept", post(accept_ride_request))
		.route("/v1/taxi/decline", post(decline_ride_request))
		.route("/v1/search", post(search))
		.route("/v1/delete_user", post(users::delete_user))
		.route("/v1/delete_car", post(cars::cars::delete_car))
		.route("/v1/taxi/isdriver/:driver_id", get(cars::taxi::is_driver))
		.route("/v1/taxi/init", post(cars::taxi::init_taxi))
		.route("/v1/taxi/images/:driver_id", get(cars::taxi::taxi_images))
		.route("/v1/taxi/price", post(taxi_price))
		// taxi verification
		.route("/v1/taxis/unverified", get(get_unverified_taxis))
		.route("/v1/:driver_id/document/:status", get(get_unverified_documents))
		.route("/v1/:driver_id/document/unverified/:type", get(get_unverified_document))
		.route("/v1/:driver_id/document/:status/:type", get(verify_document))
		.with_state(db)
		.layer(CorsLayer::permissive());

	axum::serve(listener, app).await.unwrap()
}
