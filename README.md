# CONNECTOR Backend
This is a backend server for connecting users with referral codes emphasizing concurrency and efficiency. As part of the GDLauncher tasks

## Info
This repository is the backend server for a user connect system. This allows users to sign up with a username and an optional referral code. It also allows users to login and receive real-time updates on new user signups via server sent events. You may view a live demo at
[https://gd-back-production.up.railway.app/api](https://gd-back-production.up.railway.app/api)

## Usage
To start the server, run the `main` function. The server listens on port 8080 by default. To change the port, set the `PORT` environment variable. The server requires a PostgreSQL database to be running. To configure the database connection, set the `DATABASE_URL` environment variable.

## Endpoints
The following endpoints are provided:

- POST `/api/signup`: Allows users to sign up with a username and optionally, a referral code.
- POST `/api/login`: Allows users to login with a username.
- GET `/api`:  Allows users to receive real-time updates on new user signups via server sent events.

## Dependencies
This repository was built with Rust and the Axum web framework. The following dependencies are required:
- Rust
- Axum web framework
- PostgreSQL
- SQLX

## Installation
To run this server locally, follow the steps below
- Clone the repository: `git clone https://github.com/DreadedHippy/gd-back.git`
- Move into the new folder: `cd gd-back`
- Start the server: `cargo run`


