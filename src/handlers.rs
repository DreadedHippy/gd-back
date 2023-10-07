use std::collections::HashMap;

use axum::{extract::State, Json, response::{Response, sse::Event, Sse}, TypedHeader};
use axum_extra::extract::WithRejection;
use rand::Rng;
// use serde_json::json;
use tokio_stream::{StreamExt as _, wrappers::BroadcastStream};
use futures::stream::{self, Stream};
use crate::{models::{AppState, CustomResponse, UserForCreate, User, LoginPayload}, custom_extractor::ApiError, utils::{map_err, StreamType}};
// use std::{convert::Infallible, time::Duration, ops::Deref};

// Custom response type
type ServerResponse<T> = Json<CustomResponse<T>>;

// Handler for the Server-side events
pub async fn sse_handler(
	TypedHeader(user_agent): TypedHeader<headers::UserAgent>, State(app_state): State<AppState>
) -> Sse<impl Stream<Item = Result<Event, serde_json::error::Error>>> {
	println!("`{}` connected", user_agent.as_str());
	
	// Create a broadcast stream which subscribes to the subsequent events in the app_state's tx
	let stream = BroadcastStream::new(app_state.tx.subscribe())
	.map(|r|{
		let i: StreamType = serde_json::from_str(&r.unwrap()).unwrap();
		Event::default().json_data(i)
	});

	// Let all first-time connectors receive the latest information
	let first = stream::once(async move {

		// Get the stringified json inside the Mutex
		let data = app_state.latest_info.lock().unwrap();

		// Deserialize into JSON
		let i: StreamType = serde_json::from_str(&data).unwrap();

		//? println!("{:#?}", i);

		// Send as event
		Event::default().json_data(i)
	});
	
	// Send out the latest_info to every subscriber then subsequent info in the streams
	Sse::new(first.chain(stream))
	.keep_alive(axum::response::sse::KeepAlive::new().text("keep-alive-text"))
}

// Handler for the Signup route
pub async fn handler_signup(
	State(app_state): State<AppState>,
	WithRejection(Json(user_info), _): WithRejection<Json<UserForCreate>, ApiError>
) -> Result<ServerResponse<User>, Response> {
	let rand_4 = rand::thread_rng().gen_range(1000..10000);
	let rand_5 = rand::thread_rng().gen_range(10000..100000);

	// Generate personal invite code for new users
	let personal_invite_code = format!("{}-{}-{}", user_info.username.chars().take(3).collect::<String>().to_uppercase(), rand_4, rand_5);

	// Save the user to the database, respond with appropriate messages in the event of an error
	let result = app_state.save_user(user_info, personal_invite_code).await.map_err(|e| {
		map_err(e)
	})?;

	// Create a custom JSON response, sending back user information
	let response = CustomResponse::<User>::new(
		true, 
		Some(format!("Registration successful")),
		Some(crate::models::CustomResponseData::Item(result))
	);

	// Send as 200 OK, json body
	Ok(Json(response))
}

// Handler for the "login" route
pub async fn handler_login(
	State(app_state): State<AppState>,
	WithRejection(Json(login_payload), _): WithRejection<Json<LoginPayload>, ApiError>
) -> Result<ServerResponse<User>, Response> {
	// Perform necessary database queries, respond with appropriate messages in the event of an error
	let result = app_state.get_user(login_payload.username).await.map_err(|e| {
		map_err(e)
	})?;
	
	// Create a custom JSON response, sending back user information
	let response = CustomResponse::<User>::new(
		true, 
		Some(format!("Login successful")),
		Some(crate::models::CustomResponseData::Item(result))
	);

	Ok(Json(response))

}