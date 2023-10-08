use axum::{Router, routing::{get, post}};
use tower_http::services::ServeDir;

use crate::{handlers::{sse_handler, handler_login}, models::AppState, handlers::handler_signup};

// Function returning all routes
pub fn all_routes(app_state: AppState, static_files_service: ServeDir) -> Router {
	Router::new()
		// Fallback to the static file service
		.fallback_service(static_files_service)
		// Login route
		.route("/api/login", post(handler_login))
		// Signup route
		.route("/api/register", post(handler_signup))
		// SSE and subscribers' real-time update routes
		.route("/api", get(sse_handler))
		// App state holding the pool and necessary information required by routes
		.with_state(app_state)
}