use axum::{Router, routing::{get, post}};
use tower_http::services::ServeDir;

use crate::{handlers::{sse_handler, handler_login}, models::AppState, handlers::handler_signup};

pub fn all_routes(app_state: AppState, static_files_service: ServeDir) -> Router {
	Router::new()
		.fallback_service(static_files_service)
		.route("/api/login", post(handler_login))
		.route("/api/register", post(handler_signup))
		.route("/api", get(sse_handler))
		.with_state(app_state)
}