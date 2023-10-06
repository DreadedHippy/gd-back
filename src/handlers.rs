use std::collections::HashMap;

use axum::{extract::State, Json, response::{Response, sse::Event, Sse}, TypedHeader};
use axum_extra::extract::WithRejection;
use rand::Rng;
// use serde_json::json;
use tokio_stream::{StreamExt as _, wrappers::BroadcastStream};
use futures::stream::{self, Stream};
// use std::{convert::Infallible, time::Duration, ops::Deref};


type ServerResponse<T> = Json<CustomResponse<T>>;
use crate::{models::{AppState, CustomResponse, UserForCreate, User, LoginPayload}, custom_extractor::ApiError, utils::{map_err, StreamType}};


pub async fn sse_handler(
	TypedHeader(user_agent): TypedHeader<headers::UserAgent>, State(app_state): State<AppState>
) -> Sse<impl Stream<Item = Result<Event, serde_json::error::Error>>> {
	println!("`{}` connected", user_agent.as_str());
	
	let stream = BroadcastStream::new(app_state.tx.subscribe())
	.map(|r|{
		let i: StreamType = serde_json::from_str(&r.unwrap()).unwrap();
		Event::default().json_data(i)
	});

	let first = stream::once(async move {
		let data = app_state.latest_info.lock().unwrap();
		let i: StreamType = serde_json::from_str(&data).unwrap();
		//? println!("{:#?}", i);
		Event::default().json_data(i)
	});
	
	Sse::new(first.chain(stream))
	.keep_alive(axum::response::sse::KeepAlive::new().text("keep-alive-text"))
}

pub async fn handler_signup(
	State(app_state): State<AppState>,
	WithRejection(Json(user_info), _): WithRejection<Json<UserForCreate>, ApiError>
	// Json(user_info): Json<UserForCreate>
) -> Result<ServerResponse<User>, Response> {
	let rand_4 = rand::thread_rng().gen_range(1000..10000);
	let rand_5 = rand::thread_rng().gen_range(10000..100000);

	let personal_invite_code = format!("{}-{}-{}", user_info.username.chars().take(3).collect::<String>().to_uppercase(), rand_4, rand_5);
	let result = app_state.save_user(user_info, personal_invite_code).await.map_err(|e| {
		map_err(e)
	})?;

	let response = CustomResponse::<User>::new(
		true, 
		Some(format!("Registration successful")),
		Some(crate::models::CustomResponseData::Item(result))
	);

	Ok(Json(response))
}


pub async fn handler_login(
	State(app_state): State<AppState>,
	WithRejection(Json(login_payload), _): WithRejection<Json<LoginPayload>, ApiError>
) -> Result<ServerResponse<User>, Response> {
	let result = app_state.get_user(login_payload.username).await.map_err(|e| {
		map_err(e)
	})?;

	let response = CustomResponse::<User>::new(
		true, 
		Some(format!("Login successful")),
		Some(crate::models::CustomResponseData::Item(result))
	);

	Ok(Json(response))

}