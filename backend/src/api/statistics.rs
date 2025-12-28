use axum::{
    extract::State,
    http::StatusCode,
    response::Json,
};
use crate::{
    models::{StatisticsResponse, ErrorResponse},
    state::AppState,
};

pub async fn get_statistics(
    State(state): State<AppState>,
) -> Result<Json<StatisticsResponse>, (StatusCode, Json<ErrorResponse>)> {
    match state.get_statistics().await {
        Ok(stats) => Ok(Json(stats)),
        Err(e) => {
            tracing::error!("Failed to get statistics: {}", e);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse::new("statistics_error", format!("Failed to get statistics: {}", e))),
            ))
        }
    }
}

