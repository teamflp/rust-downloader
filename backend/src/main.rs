mod api;
mod models;
mod state;
mod validation;
mod db;

use axum::{
    routing::{get, post, delete},
    Router,
    http::{header, Method},
};
use tower_http::{
    cors::{CorsLayer, Any},
    trace::TraceLayer,
};
// use tower_governor::{
//     // governor::GovernorConfigBuilder,
//     // GovernorLayer,
// };
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
use state::AppState;
use db::Database;
use std::net::SocketAddr;
// use std::sync::Arc;

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

    // TODO: Configure rate limiting with proper IP extraction
    // Currently disabled in development due to IP extraction issues
    // let governor_conf = Arc::new(
    //     GovernorConfigBuilder::default()
    //         .per_second(10)
    //         .burst_size(20)
    //         .finish()
    //         .unwrap(),
    // );

    // Configure CORS
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods([Method::GET, Method::POST, Method::DELETE, Method::OPTIONS])
        .allow_headers([header::CONTENT_TYPE, header::AUTHORIZATION]);

    // Build router
    let app = Router::new()
        .route("/api/downloads", post(api::create_download))
        .route("/api/downloads", get(api::list_downloads))
        .route("/api/downloads/:id", get(api::get_download))
        .route("/api/downloads/:id", delete(api::delete_download))
        .route("/health", get(health_check))
        .layer(cors)
        .layer(TraceLayer::new_for_http())
        // .layer(GovernorLayer {
        //     config: governor_conf,
        // })
        .with_state(state);

    // Start server
    let addr = SocketAddr::from(([0, 0, 0, 0], 3001));
    tracing::info!("Server listening on {}", addr);
    tracing::info!("Rate limiting temporarily disabled for development");
    tracing::info!("URL validation enabled");
    
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn health_check() -> &'static str {
    "OK"
}
