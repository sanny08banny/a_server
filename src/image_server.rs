use std::{
	fs::File,
	io::{BufReader, Read},
};

use axum::{extract::Path, response::Response};
use hyper::Body;

pub async fn image_handler(Path((owner_id, car_id, image)): Path<(String, String, String)>) -> Response<Body> {
	let path = format!("images/{}/{}/{}", owner_id, car_id, image);
	let h = File::open(path).expect("file not found");
	let mut buf_reader = BufReader::new(h);
	let mut contents = Vec::new();
	buf_reader.read_to_end(&mut contents).unwrap();
	Response::new(Body::from(contents))
}
