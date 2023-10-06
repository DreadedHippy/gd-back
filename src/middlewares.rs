// use axum::{middleware::Next, Json, extract::rejection::JsonRejection};

// pub async fn handler<T>(
// 	result: Result<Json<T>, JsonRejection>,
// 	next: Next<T>
// ) -> Result<Response> {
// 	match result {
// 			// if the client sent valid JSON then we're good
// 			Ok(Json(payload)) => Ok(Json(json!({ "payload": payload }))),

// 			Err(err) => match err {
// 					JsonRejection::JsonDataError(err) => {
// 							Err(serde_json_error_response(err))
// 					}
// 					JsonRejection::JsonSyntaxError(err) => {
// 							Err(serde_json_error_response(err))
// 					}
// 					// handle other rejections from the `Json` extractor
// 					JsonRejection::MissingJsonContentType(_) => Err((
// 							StatusCode::BAD_REQUEST,
// 							"Missing `Content-Type: application/json` header".to_string(),
// 					)),
// 					JsonRejection::BytesRejection(_) => Err((
// 							StatusCode::INTERNAL_SERVER_ERROR,
// 							"Failed to buffer request body".to_string(),
// 					)),
// 					// we must provide a catch-all case since `JsonRejection` is marked
// 					// `#[non_exhaustive]`
// 					_ => Err((
// 							StatusCode::INTERNAL_SERVER_ERROR,
// 							"Unknown error".to_string(),
// 					)),
// 			},
// 	}
// }