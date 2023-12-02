use std::{
    fs::File,
    io::{BufReader, Read},
};

use axum::{extract::{Path}, response::Response};
use hyper::Body;

pub async fn image_handler(owner_id:Path<String>,car_id:Path<String>,image:Path<String>) -> Response<Body> {
    let owner_id = owner_id.0;
    let car_id = car_id.0;
    let image = image.0;
    let path = format!("images/{}/{}/{}", owner_id, car_id, image);
    let h = File::open(path).expect("file not found");
    let mut buf_reader = BufReader::new(h);
    let mut contents = Vec::new();
    buf_reader.read_to_end(&mut contents).unwrap();
    Response::new(Body::from(contents))
}
