use axum::{extract::State, http::StatusCode, Json, response::{Response, IntoResponse, sse::Event, Sse}, TypedHeader};
use rand::Rng;
use tokio_stream::{StreamExt as _, wrappers::BroadcastStream};
use futures::stream::{self, Stream};
use std::{convert::Infallible, time::Duration};


type ServerResponse<T> = Json<CustomResponse<T>>;
use crate::models::{AppState, CustomResponse, UserForCreate, User, LoginPayload};


pub async fn sse_handler(
	TypedHeader(user_agent): TypedHeader<headers::UserAgent>, State(app_state): State<AppState>
) -> Sse<impl Stream<Item = Result<Event, serde_json::error::Error>>> {
	println!("`{}` connected", user_agent.as_str());

	// A `Stream` that repeats an event every second
	// let stream = stream::repeat_with(|| Event::default().data("hi!"))
	// 		.map(Ok)
	// 		.throttle(Duration::from_secs(1));

	let stream = BroadcastStream::new(app_state.tx.subscribe())
		.map(|i| Event::default().json_data(i.unwrap()));
	// let noob_stream = stream::
	// let event = Event::

	// Sse::new(stream).keep_alive(
	// 		axum::response::sse::KeepAlive::new()
	// 				.interval(Duration::from_secs(1))
	// 				.text("keep-alive-text"),
	// )
	Sse::new(stream)
	.keep_alive(axum::response::sse::KeepAlive::new().text("keep-alive-text"))
}

pub async fn handler_signup( State(app_state): State<AppState>, Json(user_info): Json<UserForCreate>) -> Result<ServerResponse<User>, Response> {
	let rand_4 = rand::thread_rng().gen_range(1000..10000);
	let rand_5 = rand::thread_rng().gen_range(10000..100000);

	let personal_invite_code = format!("{}-{}-{}", user_info.username.chars().take(3).collect::<String>(), rand_4, rand_5);

	let result = app_state.save_user(user_info, personal_invite_code).await.map_err(|e| {
		println!("{:#?}", e);
		(StatusCode::INTERNAL_SERVER_ERROR, "An error occurred while signing you up").into_response()
	})?;

	let response = CustomResponse::<User>::new(
		true, 
		Some(format!("Registration successful")),
		Some(crate::models::CustomResponseData::Item(result))
	);

	Ok(Json(response))
}


pub async fn handler_login( State(app_state): State<AppState>, Json(login_payload): Json<LoginPayload>) -> Result<ServerResponse<User>, Response> {
	let result = app_state.get_user(login_payload.username).await.map_err(|e| {
		println!("{:#?}", e);
		(StatusCode::INTERNAL_SERVER_ERROR, "An error occurred while logging you in").into_response()
	})?;

	let response = CustomResponse::<User>::new(
		true, 
		Some(format!("Login successful")),
		Some(crate::models::CustomResponseData::Item(result))
	);

	Ok(Json(response))

}