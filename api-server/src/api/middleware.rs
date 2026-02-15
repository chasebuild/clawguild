use axum::{body::Body, extract::State, http::Request, middleware::Next, response::Response};

use crate::api::errors::AppError;
use crate::api::handlers::AppState;

pub async fn require_api_key(
    State(state): State<AppState>,
    req: Request<Body>,
    next: Next,
) -> Result<Response, AppError> {
    if let Some(expected) = &state.api_key {
        let provided = req
            .headers()
            .get("x-api-key")
            .and_then(|value| value.to_str().ok());

        if provided != Some(expected.as_str()) {
            return Err(AppError::Unauthorized);
        }
    }

    Ok(next.run(req).await)
}
