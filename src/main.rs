use std::sync::Arc;
pub mod services {
    pub mod service;
}
pub mod repositories {
    pub mod database;
}
pub mod handlers {
    pub mod handler;
}

use axum::{Router, routing::get};
use handlers::handler::Appstate;
use handlers::handler::handle_batch_file;
use handlers::handler::handle_get_api;
use repositories::database::SqlxRepository;
use services::service::Service;
use sqlx::mysql::MySqlPoolOptions;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenvy::dotenv().ok();
    let database_url = std::env::var("DATABASE_URL")?;

    let pool = MySqlPoolOptions::new()
        .max_connections(100)
        .connect(&database_url)
        .await?;

    let repo = Arc::new(SqlxRepository::new(pool));

    let service = Arc::new(Service::new(repo));

    let state = Appstate { service: service };

    let shared_state = Arc::new(state);

    let app = Router::new()
        .route("/", get(handle_get_api))
        .route("/batch", get(handle_batch_file))
        .with_state(shared_state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await;

    Ok(())
}
