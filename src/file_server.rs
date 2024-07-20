use std::{
	fs::File,
	io::{BufReader, Read},
};

use axum::{extract::Path, response::Response};
use hyper::Body;
// parent_folder images or docs
pub async fn file_handler(extract: Path<(String, String, String, String, String)>) -> Response<Body> {
	let Path((parent_folder, vehicle_category, user_id, car_id, file)) = extract;
	let path = format!("{}/{}/{}/{}/{}", parent_folder, vehicle_category, user_id, car_id, file);
	let h = File::open(path).expect("file not found");
	let mut buf_reader = BufReader::new(h);
	let mut contents = Vec::new();
	buf_reader.read_to_end(&mut contents).unwrap();
	Response::new(Body::from(contents))
}
