use axum::routing::get;
use axum::Router;
use tower_http::trace;
use tower_http::trace::TraceLayer;
use tracing::Level;

use crate::calendar::generate_calendar;

mod calendar;
mod error;

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();

    tracing_subscriber::fmt()
        .with_target(false)
        .compact()
        .init();

    let app = Router::new()
        .route("/{location}/{me}", get(generate_calendar)).layer(
        TraceLayer::new_for_http()
            .make_span_with(trace::DefaultMakeSpan::new()
                .level(Level::INFO))
            .on_response(trace::DefaultOnResponse::new()
                .level(Level::INFO)),
    );

    let listener = tokio::net::TcpListener::bind("0.0.0.0:8000").await.unwrap();
    tracing::info!("listening on 0.0.0.0:8000");
    axum::serve(listener, app).await
        .unwrap();
}
