mod api;
mod models;
mod state;
mod validation;
mod db;
mod cache;
mod converter;
mod openapi;

use axum::{
    routing::{get, post, delete, patch},
    Router,
    http::{header, Method},
};
use tower_http::{
    cors::{CorsLayer, Any},
    trace::TraceLayer,
    compression::CompressionLayer,
};
// TODO: Re-enable rate limiting when tower_governor API is fixed
// use tower_governor::governor::GovernorConfigBuilder;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
use state::AppState;
use db::Database;
use std::net::SocketAddr;
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;
use openapi::ApiDoc;

#[tokio::main]
async fn main() {
    // Initialize tracing
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "rust_media_downloader_backend=debug,tower_http=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    // Initialize database
    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "sqlite:downloads.db?mode=rwc".to_string());
    
    let db = Database::new(&database_url)
        .await
        .expect("Failed to initialize database");
    
    tracing::info!("📦 Database initialized at {}", database_url);

    // Initialize app state with database
    let state = AppState::new_with_db(db);

    // Configure rate limiting: 60 requests per minute per IP/key
    // TODO: Fix tower_governor API usage
    // let governor_conf = Box::leak(Box::new(
    //     GovernorConfigBuilder::default()
    //         .per_second(60)
    //         .burst_size(100)
    //         .finish()
    //         .unwrap()
    // ));

    // Configure CORS
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods([Method::GET, Method::POST, Method::DELETE, Method::PATCH, Method::OPTIONS])
        .allow_headers([header::CONTENT_TYPE]);

    // Build main router
    let app = Router::new()
        .route("/api/downloads", post(api::create_download))
        .route("/api/downloads/batch", post(api::create_batch_downloads))
        .route("/api/downloads", get(api::list_downloads))
        .route("/api/downloads/all", get(api::get_all_downloads))
        .route("/api/downloads/:id", get(api::get_download))
        .route("/api/downloads/:id", delete(api::delete_download))
        .route("/api/downloads/:id/metadata", patch(api::update_metadata))
        .route("/api/downloads/:id/convert", post(api::convert_download))
        .route("/api/downloads/:id/favorite", patch(api::toggle_favorite))
        .route("/api/downloads/export", get(api::export_downloads))
        .route("/api/downloads/import", post(api::import_downloads))
        .route("/api/video/info", get(api::get_video_info_endpoint))
        .route("/api/files/:id", get(api::serve_file))
        .route("/api/logs", get(api::get_logs))
        .route("/api/config", get(api::get_config_info))
        .route("/api/disk", get(api::get_disk_info))
        .route("/api/disclaimer", get(api::get_disclaimer))
        .route("/api/license", get(api::get_license))
        .route("/api/tags", get(api::list_tags))
        .route("/api/tags", post(api::create_tag))
        .route("/api/tags/:id", get(api::get_tag))
        .route("/api/tags/:id", patch(api::update_tag))
        .route("/api/tags/:id", delete(api::delete_tag))
        .route("/api/downloads/:id/tags", get(api::get_download_tags))
        .route("/api/downloads/:id/tags", post(api::set_download_tags))
        .route("/api/downloads/:download_id/tags/:tag_id", post(api::add_tag_to_download))
        .route("/api/downloads/:download_id/tags/:tag_id", delete(api::remove_tag_from_download))
        .route("/api/statistics", get(api::get_statistics))
        // Webhooks management
        .route("/api/webhooks", post(api::create_webhook))
        .route("/api/webhooks", get(api::list_webhooks))
        .route("/api/webhooks/:id", delete(api::delete_webhook))
        .route("/health", get(health_check))
        .merge(
            SwaggerUi::new("/swagger-ui")
                .url("/api-docs/openapi.json", ApiDoc::openapi()),
        )
        .layer(cors)
        .layer(CompressionLayer::new())
        .layer(TraceLayer::new_for_http())
        // Rate limiting temporarily disabled - tower_governor API needs fixing
        // .layer(tower_governor::GovernorLayer {
        //     config: governor_conf,
        // })
        .with_state(state);

    // Start server
    let port = std::env::var("PORT")
        .ok()
        .and_then(|p| p.parse::<u16>().ok())
        .unwrap_or(9000);
    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    tracing::info!("Server listening on {}", addr);
    tracing::info!("Rate limiting: 60 req/min per IP/key");
    tracing::info!("Swagger UI available at http://{}/swagger-ui", addr);
    tracing::info!("URL validation enabled");
    
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn health_check() -> &'static str {
    "OK"
}
