use axum::{body::Body, extract::Path, response::IntoResponse};
use hyper::StatusCode;
use tokio::fs::File;
use tokio_util::io::ReaderStream;

// parent_folder images or docs
pub async fn file_handler(extract: Path<(String, String)>) -> impl IntoResponse {
	let Path((driver_id, file)) = extract;
	let path = format!("images/taxi/{}/{}", driver_id, file);

	match read_file_stream(&path).await {
		Some(stream) => (StatusCode::OK, Body::from_stream(stream)),
		None => (StatusCode::OK, Body::empty()),
	}
}

pub async fn get_car(path: Path<(String, String, String)>) -> impl IntoResponse {
	// owner_id,car_id,filename
	let Path((owner_id, car_id, filename)) = path;
	let path = format!("images/car_hire/{}/{}/{}", owner_id, car_id, filename);
	match read_file_stream(&path).await {
		Some(stream) => (StatusCode::OK, Body::from_stream(stream)),
		None => (StatusCode::OK, Body::empty()),
	}
}

pub async fn read_file_stream(path: &str) -> Option<ReaderStream<File>> {
	File::open(path).await.map(|f| ReaderStream::new(f)).ok()
}
