use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::Json,
};
use crate::{
    models::{Webhook, CreateWebhookRequest, ErrorResponse},
    state::AppState,
};
use url::Url;
use base64::Engine;

#[utoipa::path(
    post,
    path = "/api/webhooks",
    tag = "webhooks",
    request_body = CreateWebhookRequest,
    responses(
        (status = 200, description = "Webhook created successfully", body = Webhook),
        (status = 400, description = "Bad request", body = ErrorResponse),
    ),
)]
pub async fn create_webhook(
    State(state): State<AppState>,
    Json(request): Json<CreateWebhookRequest>,
) -> Result<Json<Webhook>, (StatusCode, Json<ErrorResponse>)> {
    // Validate URL
    if let Err(_) = Url::parse(&request.url) {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse::new("invalid_url", "Invalid webhook URL")),
        ));
    }

    // Store webhook
    match state.create_webhook(&request.url, &request.events, request.secret.as_deref()).await {
        Ok(webhook) => Ok(Json(webhook)),
        Err(e) => {
            tracing::error!("Failed to create webhook: {}", e);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse::new("creation_failed", &format!("Failed to create webhook: {}", e))),
            ))
        }
    }
}

#[utoipa::path(
    get,
    path = "/api/webhooks",
    tag = "webhooks",
    responses(
        (status = 200, description = "List of webhooks", body = Vec<Webhook>),
        (status = 500, description = "Internal server error", body = ErrorResponse),
    ),
)]
pub async fn list_webhooks(
    State(state): State<AppState>,
) -> Result<Json<Vec<Webhook>>, (StatusCode, Json<ErrorResponse>)> {
    match state.get_all_webhooks().await {
        Ok(webhooks) => {
            // Hide secrets in response
            let webhooks: Vec<Webhook> = webhooks.into_iter().map(|mut wh| {
                wh.secret = wh.secret.map(|_| "***".to_string());
                wh
            }).collect();
            Ok(Json(webhooks))
        }
        Err(e) => {
            tracing::error!("Failed to get webhooks: {}", e);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse::new("fetch_error", &format!("Failed to fetch webhooks: {}", e))),
            ))
        }
    }
}

#[utoipa::path(
    delete,
    path = "/api/webhooks/{id}",
    tag = "webhooks",
    params(
        ("id" = String, Path, description = "Webhook ID"),
    ),
    responses(
        (status = 200, description = "Webhook deleted successfully"),
        (status = 404, description = "Webhook not found", body = ErrorResponse),
    ),
)]
pub async fn delete_webhook(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<ErrorResponse>)> {
    match state.delete_webhook(&id).await {
        Ok(true) => Ok(Json(serde_json::json!({"success": true}))),
        Ok(false) => Err((
            StatusCode::NOT_FOUND,
            Json(ErrorResponse::new("not_found", "Webhook not found")),
        )),
        Err(e) => {
            tracing::error!("Failed to delete webhook: {}", e);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse::new("delete_failed", &format!("Failed to delete webhook: {}", e))),
            ))
        }
    }
}

/// Trigger webhooks for a given event type
pub async fn trigger_webhooks(
    state: &AppState,
    event_type: &str,
    event_data: serde_json::Value,
) {
    match state.get_active_webhooks_for_event(event_type).await {
        Ok(webhooks) => {
            use crate::models::WebhookEvent;
            let webhook_event = WebhookEvent {
                event_type: event_type.to_string(),
                timestamp: chrono::Utc::now(),
                data: event_data,
            };

            for webhook in webhooks {
                let state_clone = state.clone();
                let webhook_clone = webhook.clone();
                let event_clone = webhook_event.clone();
                
                tokio::spawn(async move {
            if let Err(e) = send_webhook(&state_clone, &webhook_clone, &event_clone).await {
                tracing::error!("Failed to send webhook {}: {}", webhook_clone.url, e);
            }
                });
            }
        }
        Err(e) => {
            tracing::error!("Failed to get webhooks for event {}: {}", event_type, e);
        }
    }
}

async fn send_webhook(
    state: &AppState,
    webhook: &Webhook,
    event: &crate::models::WebhookEvent,
) -> anyhow::Result<()> {
    let client = reqwest::Client::new();
    let mut request = client.post(&webhook.url).json(event);

    // Add HMAC signature if secret is configured
    if let Some(secret) = &webhook.secret {
        use hmac::{Hmac, Mac};
        use sha2::Sha256;
        type HmacSha256 = Hmac<Sha256>;
        
        let event_json = serde_json::to_string(event)?;
        let mut mac = HmacSha256::new_from_slice(secret.as_bytes())
            .map_err(|e| anyhow::anyhow!("HMAC error: {}", e))?;
        mac.update(event_json.as_bytes());
        let signature = base64::engine::general_purpose::STANDARD.encode(mac.finalize().into_bytes());
        
        request = request.header("X-Webhook-Signature", signature);
    }

    let response = request.send().await?;
    
    if response.status().is_success() {
        // Update last triggered timestamp
        let _ = state.update_webhook_last_triggered(&webhook.id).await;
    } else {
        anyhow::bail!("Webhook returned status: {}", response.status());
    }

    Ok(())
}

