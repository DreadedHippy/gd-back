use std::collections::HashMap;
use std::env;

use anyhow::Ok;
use anyhow::Result;
use axum::Json;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::response::Response;
use serde_json::json;
use sqlx::{PgPool, Pool, Postgres};

use crate::models::User;

pub type StreamType = (Vec<User>, HashMap<String, Vec<String>>);
pub async fn connect_to_postgres() -> Result<Pool<Postgres>> {
	let pool = PgPool::connect(&env::var("DATABASE_URL")?).await?;

	// let query = r#"
	// 	DROP TABLE IF EXISTS users
	// "#;


	let query = r#"
		CREATE TABLE IF NOT EXISTS users (
			id SERIAL PRIMARY KEY,
			username VARCHAR(50) UNIQUE NOT NULL,
			referral_code VARCHAR(50),
			personal_invite_code VARCHAR(50) UNIQUE,
			FOREIGN KEY (referral_code) REFERENCES users(personal_invite_code) ON DELETE SET NULL
		);
	"#;


	let record: sqlx::query::Query<'_, Postgres, sqlx::postgres::PgArguments> = sqlx::query(query);
	
	let _ = record.execute(&pool).await?;

	// let query2 = r#"
	// INSERT INTO users (username, referral_code, personal_invite_code)
	// 	VALUES ('Genesis', '', '')
	// "#;
	
	// let record: sqlx::query::Query<'_, Postgres, sqlx::postgres::PgArguments> = sqlx::query(query2);

	// let _ = record.execute(&pool).await?;
	
	Ok(pool)
}

pub fn map_err(e: sqlx::Error) -> Response {
	let error = match e {
		sqlx::Error::Database(err) => {
			println!("\n Database Error ->> {:#?} \n", err);
			let f = String::from(err.code().as_deref().unwrap_or_default().to_owned());
			let f = f.as_str();
			let g = match f {
				// REFERRAL CODE IS INVALID
				"23503" => {
					let payload = json!({
						"message": "`referral_code` is invalid",
						"origin": "Postgres error"
					});
					
					(StatusCode::BAD_REQUEST, Json(payload)).into_response()
				},

				// USER ALREADY EXISTS IN THE DATABASE
				"23505" => {
					let payload = json!({
						"message": "User already exists",
						"origin": "Postgres error"
					});						
					(StatusCode::CONFLICT, Json(payload)).into_response()
				},
				_ => {
					(StatusCode::INTERNAL_SERVER_ERROR, "An error occurred while logging you in").into_response()
				}
			};

			g
		},
		// COULD NOT FIND USER
		sqlx::Error::RowNotFound => {
			let payload = json!({
				"message": "User not found",
				"origin": "Postgres error"
			});

			(StatusCode::NOT_FOUND, Json(payload)).into_response()
		},
		_ => {
			println!("\n Error ->> {:#?} \n", e);
			(StatusCode::INTERNAL_SERVER_ERROR, "An error occurred while logging you in").into_response()
		}
	};

	error

}

pub async fn get_initial_info() ->  String {
	let pool = connect_to_postgres().await.unwrap();
	let mut map:HashMap<String, Vec<String>> = HashMap::new();

	let q = r#"
		SELECT *
		FROM users
	"#;

	let record = sqlx::query_as::<_, User>(q);

	let users = record
		.fetch_all(&pool)
		.await.unwrap();


	users.iter().for_each(|u| {
		let _ = map.entry(u.personal_invite_code.clone()).or_insert(vec![]);
		map.entry(u.referral_code.clone()).or_insert(vec![]).push(u.username.clone());
	});

	serde_json::to_string(&(users, map)).unwrap()

}