use axum::Json;

async fn location(s: Json<String>) {
	let s = s.0;
}
