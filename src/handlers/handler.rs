use crate::repositories;
use crate::services::service::Service;
use axum::{Json, extract::State, http::StatusCode, response::IntoResponse};
use std::sync::Arc;

#[derive(Clone)]
pub struct Appstate {
    pub service: Arc<Service>,
}

pub async fn handle_get_api(
    State(state): State<Arc<Appstate>>,
) -> Result<Json<Vec<repositories::database::LegoSet>>, StatusCode> {
    let result = state
        .service
        .get()
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Json(result))
}

pub async fn handle_batch_file(State(state): State<Arc<Appstate>>) -> impl IntoResponse {
    state.service.batch().await;
}
