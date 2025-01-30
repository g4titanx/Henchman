// The config file for this app acts as a typical .env file. Alot of API keys are needed for this agent

use serde::{Deserialize, Serialize};

/// Path to the configuration file that contains all settings
const CONFIG: &str = include_str!("../config.toml");

/// Configuration for the Hyperbolic LLM service.
/// Controls the behavior of text generation through model parameters
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct HyperbolicConfig {
    /// Model identifier to use for text generation (e.g., "meta-llama/Meta-Llama-3.1-70B-Instruct")
    pub model: String,
    /// Maximum number of tokens to generate in a response
    pub max_tokens: u32,
    /// Controls randomness in text generation (0.0-1.0)
    /// Higher values make output more diverse but less focused
    pub temperature: f32,
    /// Nucleus sampling parameter (0.0-1.0)
    /// Controls cumulative probability threshold for token selection
    pub top_p: f32,
    /// Number of highest probability tokens to consider for sampling
    pub top_k: u32,
}

/// Main configuration structure for the AI agent.
/// Loaded from config.toml and controls all aspects of the agent's behavior.
/// This file serves as a consolidated .env file containing necessary API keys and settings.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct Config {
    /// Twitter username for the agent's account
    pub x_username: String,
    /// OAuth consumer key from Twitter developer portal
    pub x_consumer_key: String,
    /// OAuth consumer secret from Twitter developer portal
    pub x_consumer_secret: String,
    /// OAuth access token for agent's Twitter account
    pub x_access_token: String,
    /// OAuth access token secret for agent's Twitter account
    pub x_access_token_secret: String,
    /// URL for Ethereum RPC endpoint
    /// Used for block number tracking in timelock mechanism
    pub eth_rpc_url: String,
    /// Path to RocksDB storage for key-value data
    pub kv_db_path: String,
    /// Minimum significance score (1-10) required to store a memory
    /// Higher values mean only more significant events are remembered
    pub min_storing_memory_score: u16,
    /// Minimum significance score (1-10) required to post a tweet
    /// Higher values result in fewer but more significant tweets
    pub min_posting_score: u16,
    /// Maximum number of mentions to process in one run
    pub max_num_mentions: usize,
    /// Maximum number of timeline tweets to process in one run
    pub max_timeline_tweets: usize,
    /// Number of relevant long-term memories to retrieve for context
    pub num_long_term_memories: u64,
    /// Number of recent posts to use for context in tweet generation
    pub num_recent_posts: usize,
    /// Minimum score required to respond to a mention (1-10)
    /// Controls how selective the agent is in responding
    pub min_mention_score: u8,
    /// Optional range (min, max) in seconds for sleep between scrolling sessions
    /// Used to simulate natural Twitter usage patterns
    pub scroll_sleep: Option<(u64, u64)>,
    /// Optional range (min, max) in seconds for duration of scrolling sessions
    pub scroll_duration: Option<(u64, u64)>,
    /// Optional range (min, max) in seconds for sleep between agent runs
    pub run_sleep: Option<(u64, u64)>,
    /// Time in seconds after which to release account credentials
    pub release_credentials: u64,
    /// Configuration for the Hyperbolic LLM service
    pub hyperbolic: HyperbolicConfig,
    /// URL for the Qdrant vector database instance
    pub vector_db_url: String,
}

impl Config {
    /// Loads configuration from the embedded config.toml file
    pub fn load() -> Self {
        toml::from_str(CONFIG).expect("Unable to parse config.toml")
    }
}

#[test]
fn test_load() {
    Config::load();
}
