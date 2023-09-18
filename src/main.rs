use axum::{ extract::{Json, Path,Multipart}, routing::{get, Router, 
    post}, body::{Bytes,Body}
};
use hyper::Response; use serde_json::Value; use std::{
    fs::{self, File}, io::{BufReader, Read}, net::SocketAddr,
};
use tower_http::cors::CorsLayer;

#[tokio::main]
async fn main() { let addr = "0.0.0.0:4000"; let app = Router::new()
        .route("/houses", get(handler)) .route("/houses/:path", 
        get(image_handler)) .route("/houses/download/:path", 
        get(download_img)) .route("/houses/upload/:filename", 
        post(img_upload)) .route("/houses/mult_upload", 
        post(mult_upload)) .layer(CorsLayer::permissive());
    axum::Server::bind(&addr.trim().parse().expect("Invalid address")) 
        .serve(app.into_make_service_with_connect_info::<SocketAddr>()) 
        .await .unwrap();
}
async fn handler() -> Json<Value> { let y = 
    fs::read_to_string("test.json").expect("json file not found"); let 
    x: Value = serde_json::from_str(&y).expect("invalid json"); Json(x)
}

async fn image_handler(path: Path<String>) -> Response<Body> { 
    println!("file path: {:?}",path.0); let h = 
    File::open(path.0).expect("file not found"); let mut buf_reader = 
    BufReader::new(h); let mut contents = Vec::new(); 
    buf_reader.read_to_end(&mut contents).unwrap(); 
    Response::new(Body::from(contents))
}

async fn download_img(path: Path<String>) -> Response<Body> { let h = 
    File::open(path.0.clone()).expect("file not found"); let mut 
    buf_reader = BufReader::new(h); let mut contents = Vec::new(); 
    buf_reader.read_to_end(&mut contents).unwrap(); let attachment = 
    format!("attachment; filename={}", path.0); Response::builder()
        .header("Content-Disposition", attachment) 
        .body(Body::from(contents)) .unwrap()
}

async fn img_upload(path:Path<String>,body: Bytes){ let img = 
    image::load_from_memory(&body).unwrap(); img.save(path.0).unwrap();
}

async fn mult_upload(multipart:Multipart){ println!("Data: 
{:#?}",multipart);
}
