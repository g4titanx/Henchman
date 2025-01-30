use std::sync::{Arc, Mutex, OnceLock};

use serde::Deserialize;

use axum::http::{
    header::{ACCEPT, AUTHORIZATION, CONTENT_TYPE},
    HeaderValue, Method,
};

use axum::{extract::State, routing::post, Json, Router};

use tokio::sync::oneshot::{self, Receiver, Sender};
use tower_http::cors::CorsLayer;

/// Global storage for environment variables
/// Uses OnceLock to ensure thread-safe single initialization
pub static ENV: OnceLock<EnvVariables> = OnceLock::new();

/// Structure containing API keys required for the agent's operation
/// These are provided via HTTP endpoint after the agent starts
#[derive(Deserialize, Debug)]
pub struct EnvVariables {
    /// API key for Hyperbolic LLM service
    pub hyperbolic_api_key: String,
    /// API key for OpenAI embeddings service
    pub open_ai_api_key: String,
}

/// Starts a temporary HTTP server to receive API keys
/// This server runs until the keys are received and then shuts down gracefully
///
/// The server:
/// 1. Listens on port 6969
/// 2. Accepts POST requests with API keys
/// 3. Stores the keys in ENV
/// 4. Shuts down after successful key receipt
///
/// This approach allows secure key delivery after the agent is running
/// in its trusted execution environment.
pub async fn wait_for_api_keys() {
    tracing::info!("Waiting for api keys to be delivered");

    // Configure CORS for local development environment
    let cors = CorsLayer::new()
        .allow_origin("http://localhost:3000".parse::<HeaderValue>().unwrap())
        .allow_methods([Method::POST])
        .allow_credentials(true)
        .allow_headers([AUTHORIZATION, ACCEPT, CONTENT_TYPE]);

    // Create shutdown channel for graceful server termination
    let (shutdown_sender, shutdown_receiver): (Sender<()>, Receiver<()>) = oneshot::channel();

    // Configure router with shutdown state
    let app = Router::new()
        .route("/", post(get_env_variables))
        .layer(cors)
        .with_state(Arc::new(Mutex::new(Some(shutdown_sender))));

    // Start server and listen for connections
    let listener = tokio::net::TcpListener::bind("0.0.0.0:6969").await.unwrap();
    axum::serve(listener, app)
        .with_graceful_shutdown(async {
            shutdown_receiver.await.unwrap();
        })
        .await
        .expect("ENV server panic unexpectedly");
}

/// Handler for POST requests containing API keys.
///
/// # Arguments
/// * `shutdown_sender` - Channel sender for graceful shutdown
/// * `env_variables` - JSON payload containing API keys
///
/// The handler:
/// 1. Sets the received environment variables
/// 2. Initiates server shutdown
/// 3. Returns success/failure message
async fn get_env_variables(
    State(shutdown_sender): State<Arc<Mutex<Option<Sender<()>>>>>,
    Json(env_variables): Json<EnvVariables>,
) -> Result<String, String> {
    // Set environment variables in global storage
    ENV.set(env_variables)
        .map_err(|_| "Failed to set environment variables")?;

    tracing::info!("Successfully set the ENV variables, shutting down server");

    // Initiate graceful shutdown
    match shutdown_sender.lock() {
        Ok(mut sender) => {
            if let Some(s) = sender.take() {
                if let Err(e) = s.send(()) {
                    tracing::error!("Failed to send shutdown signal: {:?}", e);
                    return Err("Failed to shutdown server".into());
                }
            }
        }
        Err(e) => {
            tracing::error!("Failed to acquire shutdown sender lock: {}", e);
            return Err("Failed to shutdown server".into());
        }
    }

    Ok("Successfully set ENV variables".into())
}
