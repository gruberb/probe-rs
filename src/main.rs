use axum::{routing::get, Router};
use std::net::SocketAddr;
use tracing_subscriber;
mod favicon;
mod handlers;
mod image;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    let app = Router::new().route("/favicon", get(handlers::fetch_favicon));

    let addr = SocketAddr::from(([0, 0, 0, 0], 3000));
    tracing::debug!("Listening on {}", addr);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
