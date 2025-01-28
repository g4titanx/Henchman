use std::error::Error;

use config::Config;
use env::wait_for_api_keys;
use pipeline::Pipeline;
use prompts::Prompts;
use tracing_subscriber::{fmt, prelude::*, EnvFilter};

pub mod agent;
pub mod config;
pub mod db;
pub mod env;
pub mod hyperbolic;
pub mod openai;
pub mod pipeline;
pub mod prompts;
pub mod twitter;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // Quote Server logs config
    let fmt_layer = fmt::layer()
        .with_target(true)
        .with_level(true)
        .with_thread_ids(false)
        .with_thread_names(true);

    let filter_layer = EnvFilter::try_from_default_env()
        .or_else(|_| EnvFilter::try_new::<String>("Debug".into()))
        .expect("Error tracing subscriber filter layer");

    tracing_subscriber::registry()
        .with(filter_layer)
        .with(fmt_layer)
        .init();

    let prompts = Prompts::load();
    let config = Config::load();

    // First wait to be provided the api keys we need to run the AI Agen
    wait_for_api_keys().await;

    tracing::info!("Starting AI Agent");
    let mut pipeline = Pipeline::new(config, prompts).await;
    pipeline.run().await;

    Ok(())
}
