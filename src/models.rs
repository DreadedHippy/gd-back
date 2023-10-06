use std::{collections::HashMap, sync::{Arc, Mutex}};

// use axum::{response::{Response, IntoResponse}, http::StatusCode};
// use futures::channel::mpsc::Sender;
use serde::{Deserialize, Serialize};
use sqlx::{Postgres, Pool, Error, FromRow};
use tokio::sync::broadcast;

#[derive(Clone)]
pub struct AppState {
	pub pool: Pool<Postgres>,
	pub tx: broadcast::Sender<String>,
	pub latest_info: Arc<Mutex<String>>
	// pub referrals: Sender<i32>
}

#[derive(Deserialize, Serialize, Debug, FromRow, Clone)]
pub struct User {
	pub id: i32,
	pub username: String,
	pub referral_code: String,
	pub personal_invite_code: String
}

#[derive(Deserialize, Serialize)]
pub struct UserForCreate {
	pub username: String,
	pub referral_code: String
}

#[derive(Serialize)]
pub struct CustomResponse<T> {
	status: bool,
	message: Option<String>,
	data: Option<CustomResponseData<T>>
}

#[derive(Serialize)]
#[serde(untagged)]
pub enum  CustomResponseData<T> {
	Item(T),
	// Collection(Vec<T>),
	// Message(String)		
}

#[derive(Deserialize, Serialize)]
pub struct LoginPayload {
	pub username: String
}

impl AppState {
	pub async fn save_user(mut self, user: UserForCreate, personal_invite_code: String) -> Result<User, Error> {
		let q = r#"
			INSERT INTO users (username, referral_code, personal_invite_code)
			VALUES ($1, $2, $3)
			RETURNING *
		"#;

		let record = sqlx::query_as::<_, User>(q);

		let user = record
			.bind(user.username)
			.bind(user.referral_code)
			.bind(personal_invite_code)
			.fetch_one(&self.pool)
			.await?;


		tokio::spawn(async move{
			let latest_info = self.get_latest_info_from_db().await;
			let mut info_state = self.latest_info.lock().unwrap();
			*info_state = latest_info;
			self.tx.send(info_state.clone()).unwrap();
		});

		Ok(user)
	}

	
	pub async fn get_all_users(&self) -> Result<Vec<User>, Error> {
		let q = r#"
			SELECT *
			FROM users
		"#;

		let record = sqlx::query_as::<_, User>(q);

		let users = record
			.fetch_all(&self.pool)
			.await?;

		Ok(users)
	}

	pub async fn get_latest_info_from_db(&self) ->  String {
		let mut map:HashMap<String, Vec<String>> = HashMap::new();
		let users = self.get_all_users().await.expect("Failed to get all users");

		users.iter().for_each(|u| {
			let _ = map.entry(u.personal_invite_code.clone()).or_insert(vec![]);
			map.entry(u.referral_code.clone()).or_insert(vec![]).push(u.username.clone());
		});

		serde_json::to_string(&(users, map)).unwrap()

	}

	
	pub async fn get_user(self, username: String) -> Result<User, Error> {
		let q = r#"
			SELECT *
			FROM users
			WHERE username = $1
		"#;

		let record = sqlx::query_as::<_, User>(q);

		let user = record
			.bind(username)
			.fetch_one(&self.pool)
			.await?;

		Ok(user)
	}
}

impl<T> CustomResponse<T> {
    pub fn new(
			status: bool,
			message: Option<String>,
			data: Option<CustomResponseData<T>>
		) -> Self { Self { status, message, data } }
}