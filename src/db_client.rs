use tokio_postgres::{Client, NoTls};


pub async fn db_client() -> Client {
    let host = "localhost";
    let user = "ubuntu";
    let password = "new_password";
    let dbname = "ubuntu";
    let config_string = format!(
        "host={} user={} password={} dbname={}",
        host, user, password, dbname
    );
    let (client, monitor) = tokio_postgres::connect(config_string.as_str(), NoTls)
        .await
        .unwrap();

    tokio::spawn(async move {
        if let Err(e) = monitor.await {
            eprintln!("Connection error: {}", e);
        }
    });

    client
}
