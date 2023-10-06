use axum::http::{Method, header::CONTENT_TYPE};
use tokio::{stream, sync::broadcast};
use std::{net::SocketAddr, path::PathBuf, collections::HashMap};
use tower_http::{services::ServeDir, trace::TraceLayer, cors::{CorsLayer, Any}};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
use dotenv::dotenv;

use crate::{utils::connect_to_postgres, routes::all_routes, models::{AppState, User}};

mod utils;
mod routes;
mod handlers;
mod models;
mod middlewares;
pub mod custom_extractor;

#[tokio::main]
async fn main() {
    dotenv().ok();

    let pool = connect_to_postgres().await.expect("Could not connect to postgres");

    let cors = CorsLayer::new()
        .allow_methods([Method::GET, Method::POST])
        .allow_origin(Any)
        .allow_headers(Any);

    
    let (tx, _) = broadcast::channel(2);

    let app_state = AppState {
        pool,
        tx
    };


    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "example_sse=debug,tower_http=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    let assets_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("assets");

    let static_files_service = ServeDir::new(assets_dir).append_index_html_on_directories(true);

    // build our application with a route
    // let app = Router::new()
    //     .fallback_service(static_files_service)
    //     .route("/sse", get(sse_handler))
    //     .layer(TraceLayer::new_for_http())
    //     .layer(cors);

    let routes = all_routes(app_state, static_files_service)
    .layer(TraceLayer::new_for_http())
    .layer(cors);

    // run it
    let addr = SocketAddr::from(([127, 0, 0, 1], 8000));
    tracing::debug!("listening on {}", addr);
    axum::Server::bind(&addr)
        .serve(routes.into_make_service())
        .await
        .unwrap();
}




// use tokio_stream::StreamExt;

// #[tokio::main]
// async fn main() {
//     let mut stream = tokio_stream::iter(&[1, 2, 3]);

//     while let Some(v) = stream.next().await {
//         println!("GOT = {:?}", v);
//     }
// }