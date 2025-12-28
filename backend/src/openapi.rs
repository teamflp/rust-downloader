use utoipa::OpenApi;

use crate::models::{
    ErrorResponse,
    Webhook, CreateWebhookRequest, WebhookEvent,
};

#[derive(OpenApi)]
#[openapi(
    paths(
        crate::api::webhooks::create_webhook,
        crate::api::webhooks::list_webhooks,
        crate::api::webhooks::delete_webhook,
    ),
    components(schemas(
        ErrorResponse,
        Webhook,
        CreateWebhookRequest,
        WebhookEvent,
    )),
    tags(
        (name = "webhooks", description = "Gestion des webhooks"),
    ),
)]
pub struct ApiDoc;

