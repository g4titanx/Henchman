//! AI Agent for Twitter/X

use std::error::Error;
use tee_ai_agent::{config::Config, env::wait_for_api_keys, pipeline::Pipeline, prompts::Prompts};
use tracing_subscriber::{fmt, prelude::*, EnvFilter};

/// Entry point for the AI agent system.
///
/// Startup sequence:
/// 1. Initialize logging configuration
/// 2. Load prompts and configuration
/// 3. Wait for API key delivery
/// 4. Start the agent pipeline
///
/// The system uses structured logging via tracing for monitoring
/// and debugging capabilities.
#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // Configure structured logging
    let fmt_layer = fmt::layer()
        .with_target(true)
        .with_level(true)
        .with_thread_ids(false)
        .with_thread_names(true);

    // Set up logging filter from environment or default to Debug
    let filter_layer = EnvFilter::try_from_default_env()
        .or_else(|_| EnvFilter::try_new::<String>("Debug".into()))
        .expect("Error tracing subscriber filter layer");

    // Initialize the logging system
    tracing_subscriber::registry()
        .with(filter_layer)
        .with(fmt_layer)
        .init();

    // Load system configuration
    let prompts = Prompts::load();
    let config = Config::load();

    // Wait for API keys to be securely delivered
    wait_for_api_keys().await;

    tracing::info!("Starting AI Agent");

    // Initialize and run the main agent pipeline
    let mut pipeline = Pipeline::new(config, prompts).await;
    pipeline.run().await;

    Ok(())
}
