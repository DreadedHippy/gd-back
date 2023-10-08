use axum::http::Method;
use tokio::sync::broadcast;
use std::{net::SocketAddr, path::PathBuf, sync::{Arc, Mutex}};
use tower_http::{services::ServeDir, trace::TraceLayer, cors::{CorsLayer, Any}};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
use dotenv::dotenv;

use crate::{utils::{connect_to_postgres, get_initial_info}, routes::all_routes, models::AppState};

mod utils;
mod routes;
mod handlers;
mod models;
pub mod custom_extractor;

#[tokio::main]
async fn main() {
    // Load env variables
    dotenv().ok();

    // Get postgres connection pool
    let pool = connect_to_postgres().await.expect("Could not connect to postgres");

    // Define a cors middleware accepting all origins and headers
    let cors = CorsLayer::new()
        .allow_methods([Method::GET, Method::POST])
        .allow_origin(Any)
        .allow_headers(Any);

    // Create a broadcast channel for the SSE
    let (tx, _) = broadcast::channel(2);

    // Get the most recent info from the database
    let initial_info = get_initial_info().await;

    // Create a new Arc<Mutex> of this latest info
    let latest_info = Arc::new(Mutex::new(initial_info));

    // Construct the app state
    let app_state = AppState {
        pool,
        tx,
        latest_info
    };


    // Help with debugging
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "example_sse=debug,tower_http=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    // For static file serving
    let assets_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("assets");

    // Create a static file service
    let static_files_service = ServeDir::new(assets_dir).append_index_html_on_directories(true);

    // Get all routes using the `all_routes` function
    let routes = all_routes(app_state, static_files_service)
    .layer(TraceLayer::new_for_http())
    .layer(cors);

    // Specify the port to run on
    let port = 8080;

    let addr = SocketAddr::from(([0, 0, 0, 0], port));

    tracing::debug!("listening on {}", addr);
    
    // Start the server
    axum::Server::bind(&addr)
        .serve(routes.into_make_service())
        .await
        .unwrap();
}
