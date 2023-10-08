use std::{collections::HashMap, sync::{Arc, Mutex}};
use serde::{Deserialize, Serialize};
use sqlx::{Postgres, Pool, Error, FromRow};
use tokio::sync::broadcast;

// Struct defining the app state
#[derive(Clone)]
pub struct AppState {
	pub pool: Pool<Postgres>,
	pub tx: broadcast::Sender<String>,
	pub latest_info: Arc<Mutex<String>>
}

// Struct defining user stored in database
#[derive(Deserialize, Serialize, Debug, FromRow, Clone)]
pub struct User {
	pub id: i32,
	pub username: String,
	pub referral_code: String,
	pub personal_invite_code: String
}

// Struct defining fields required to create a new user
#[derive(Deserialize, Serialize)]
pub struct UserForCreate {
	pub username: String,
	pub referral_code: String
}

// Struct representing our custom JSON response
#[derive(Serialize)]
pub struct CustomResponse<T> {
	status: bool,
	message: Option<String>,
	data: Option<CustomResponseData<T>>
}

// Enum representing our custom JSON reponse `data` field
#[derive(Serialize)]
#[serde(untagged)]
pub enum  CustomResponseData<T> {
	Item(T),
	// Collection(Vec<T>),
	// Message(String)		
}

// Struct representing information required to login
#[derive(Deserialize, Serialize)]
pub struct LoginPayload {
	pub username: String
}

impl AppState {
	// Save a user to the database
	pub async fn save_user(self, user: UserForCreate, personal_invite_code: String) -> Result<User, Error> {
		let q = r#"
			INSERT INTO users (username, referral_code, personal_invite_code)
			VALUES ($1, $2, $3)
			RETURNING *
		"#;

		// Return the result mpped to User
		let record = sqlx::query_as::<_, User>(q);

		// Save user and retrieve fields
		let user = record
			.bind(user.username)
			.bind(user.referral_code)
			.bind(personal_invite_code)
			.fetch_one(&self.pool)
			.await?;


		// Update all stream listeners on a separate thread, for efficiency
		tokio::spawn(async move{
			// Get latest info from the database
			let latest_info = self.get_latest_info_from_db().await;

			// Update information inside mutex
			let mut info_state = self.latest_info.lock().unwrap();
			*info_state = latest_info;

			// Notify all subscribers of new user(real-time update)
			self.tx.send(info_state.clone()).unwrap();
		});

		// Send new user an "OK" response
		Ok(user)
	}

	// Get all users from database
	pub async fn get_all_users(&self) -> Result<Vec<User>, Error> {
		let q = r#"
			SELECT *
			FROM users
		"#;

		// Return result mapped to users
		let record = sqlx::query_as::<_, User>(q);

		// Get a vector of all users in db
		let users = record
			.fetch_all(&self.pool)
			.await?;

		Ok(users)
	}

	// Get latest info from database
	pub async fn get_latest_info_from_db(&self) ->  String {
		let mut map:HashMap<String, Vec<String>> = HashMap::new();

		// Get all users
		let users = self.get_all_users().await.expect("Failed to get all users");

		// Convert it to a form required by our listeners
		users.iter().for_each(|u| {
			let _ = map.entry(u.personal_invite_code.clone()).or_insert(vec![]);
			map.entry(u.referral_code.clone()).or_insert(vec![]).push(u.username.clone());
		});

		// Serialize the information
		serde_json::to_string(&(users, map)).unwrap()

	}

	// Get a single user
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

// Create a custom response via function
impl<T> CustomResponse<T> {
    pub fn new(
			status: bool,
			message: Option<String>,
			data: Option<CustomResponseData<T>>
		) -> Self { Self { status, message, data } }
}