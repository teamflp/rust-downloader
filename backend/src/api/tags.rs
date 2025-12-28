use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::Json,
};
use crate::{
    models::{Tag, CreateTagRequest, UpdateTagRequest, ErrorResponse},
    state::AppState,
};

pub async fn create_tag(
    State(state): State<AppState>,
    Json(request): Json<CreateTagRequest>,
) -> Result<Json<Tag>, (StatusCode, Json<ErrorResponse>)> {
    // Check if tag with same name already exists
    if let Some(_) = state.get_tag_by_name(&request.name).await {
        return Err((
            StatusCode::CONFLICT,
            Json(ErrorResponse::new("tag_exists", "A tag with this name already exists")),
        ));
    }

    match state.create_tag(request).await {
        Ok(tag) => Ok(Json(tag)),
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse::new("creation_failed", &format!("Failed to create tag: {}", e))),
        )),
    }
}

pub async fn get_tag(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<Tag>, (StatusCode, Json<ErrorResponse>)> {
    match state.get_tag(&id).await {
        Some(tag) => Ok(Json(tag)),
        None => Err((
            StatusCode::NOT_FOUND,
            Json(ErrorResponse::new("not_found", "Tag not found")),
        )),
    }
}

pub async fn list_tags(
    State(state): State<AppState>,
) -> Json<Vec<Tag>> {
    Json(state.get_all_tags().await)
}

pub async fn update_tag(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(request): Json<UpdateTagRequest>,
) -> Result<Json<Tag>, (StatusCode, Json<ErrorResponse>)> {
    let mut tag = match state.get_tag(&id).await {
        Some(t) => t,
        None => {
            return Err((
                StatusCode::NOT_FOUND,
                Json(ErrorResponse::new("not_found", "Tag not found")),
            ));
        }
    };

    if let Some(name) = request.name {
        tag.name = name;
    }
    if let Some(color) = request.color {
        tag.color = Some(color);
    }
    if let Some(category) = request.category {
        tag.category = Some(category);
    }

    match state.update_tag(tag.clone()).await {
        Ok(_) => Ok(Json(tag)),
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse::new("update_failed", &format!("Failed to update tag: {}", e))),
        )),
    }
}

pub async fn delete_tag(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<StatusCode, (StatusCode, Json<ErrorResponse>)> {
    if state.delete_tag(&id).await {
        Ok(StatusCode::NO_CONTENT)
    } else {
        Err((
            StatusCode::NOT_FOUND,
            Json(ErrorResponse::new("not_found", "Tag not found")),
        ))
    }
}

pub async fn get_download_tags(
    State(state): State<AppState>,
    Path(download_id): Path<String>,
) -> Json<Vec<Tag>> {
    Json(state.get_download_tags(&download_id).await)
}

pub async fn add_tag_to_download(
    State(state): State<AppState>,
    Path((download_id, tag_id)): Path<(String, String)>,
) -> Result<StatusCode, (StatusCode, Json<ErrorResponse>)> {
    match state.add_tag_to_download(&download_id, &tag_id).await {
        Ok(_) => Ok(StatusCode::NO_CONTENT),
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse::new("operation_failed", &format!("Failed to add tag: {}", e))),
        )),
    }
}

pub async fn remove_tag_from_download(
    State(state): State<AppState>,
    Path((download_id, tag_id)): Path<(String, String)>,
) -> Result<StatusCode, (StatusCode, Json<ErrorResponse>)> {
    match state.remove_tag_from_download(&download_id, &tag_id).await {
        Ok(_) => Ok(StatusCode::NO_CONTENT),
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse::new("operation_failed", &format!("Failed to remove tag: {}", e))),
        )),
    }
}

pub async fn set_download_tags(
    State(state): State<AppState>,
    Path(download_id): Path<String>,
    Json(tag_ids): Json<Vec<String>>,
) -> Result<StatusCode, (StatusCode, Json<ErrorResponse>)> {
    match state.set_download_tags(&download_id, &tag_ids).await {
        Ok(_) => Ok(StatusCode::NO_CONTENT),
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse::new("operation_failed", &format!("Failed to set tags: {}", e))),
        )),
    }
}

