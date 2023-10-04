use std::env;

use anyhow::Ok;
use anyhow::Result;
// use axum::response::Response;
// use sqlx::postgres::PgConnectOptions;
use sqlx::{PgPool, Pool, Postgres};

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


	let record = sqlx::query(query);

	// let query2 = r#"
	// INSERT INTO users (username, referral_code, personal_invite_code)
	// 	VALUES ('u', '', '')
	// "#;
	
	// let record = sqlx::query(query2);

	let _ = record.execute(&pool).await?;
	
	Ok(pool)
}