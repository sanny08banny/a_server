use std::{
    fs::File,
    io::{BufReader, Read},
};

use axum::{extract::Json, response::Response};
use hyper::Body;

#[derive(serde::Deserialize, Debug)]
pub struct CarImage {
    car_id: String,
    owner_id: String,
    image: String,
}

pub async fn image_handler(arr_req: Json<CarImage>) -> Response<Body> {
    let s = arr_req.0;
    let path = format!("images/{}/{}/{}", s.owner_id, s.car_id, s.image);
    let h = File::open(path).expect("file not found");
    let mut buf_reader = BufReader::new(h);
    let mut contents = Vec::new();
    buf_reader.read_to_end(&mut contents).unwrap();
    Response::new(Body::from(contents))
}
