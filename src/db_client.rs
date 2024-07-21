use std::sync::Arc;

use tokio_postgres::NoTls;

#[derive(Debug, Clone)]
pub struct DbClient(Arc<tokio_postgres::Client>);

impl std::ops::Deref for DbClient {
	type Target = tokio_postgres::Client;

	fn deref(&self) -> &Self::Target {
		&self.0
	}
}

pub(super) async fn db_client() -> DbClient {
	let host = "localhost";
	let user = "ubuntu";
	let password = "new_password";
	let dbname = "ubuntu";
	let config_string = format!("host={} user={} password={} dbname={}", host, user, password, dbname);
	let (client, monitor) = tokio_postgres::connect(config_string.as_str(), NoTls).await.unwrap();

	tokio::spawn(async move {
		if let Err(e) = monitor.await {
			eprintln!("Connection error: {}", e);
		}
	});

	DbClient(Arc::new(client))
}
