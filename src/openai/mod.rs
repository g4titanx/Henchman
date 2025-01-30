use anyhow::{anyhow, Result};
use reqwest::Client;

use self::api_types::ApiResponse;
mod api_types;

/// Client for interacting with OpenAI's API, specifically for generating text embeddings.
/// Text embeddings are vector representations of text that capture semantic meaning,
/// allowing for similarity comparisons and semantic search.
pub struct OpenAIClient {
    /// Base URL for the OpenAI API
    base_url: String,
    /// API key for authentication with OpenAI
    open_ai_api_key: String,
    /// HTTP client for making requests
    client: Client,
}

impl OpenAIClient {
    /// Creates a new OpenAIClient instance
    pub fn new(open_ai_api_key: String, base_url: String) -> Self {
        let client = Client::new();
        Self {
            base_url,
            open_ai_api_key,
            client,
        }
    }

    /// Generates vector embeddings for the provided text using OpenAI's text embedding model.
    /// Returns embeddings that can be used for semantic similarity search and other NLP tasks.
    ///
    /// # Note
    /// Uses the "text-embedding-3-small" model which generates consistent dimension vectors
    pub async fn get_text_embedding(&self, text: &str) -> Result<ApiResponse> {
        let url = format!("{}/embeddings", self.base_url);

        let body = serde_json::json!({
            "input": text,
            "model": "text-embedding-3-small"
        });

        self.client
            .post(&url)
            .header("Content-Type", "application/json")
            .header("Authorization", format!("Bearer {}", self.open_ai_api_key))
            .body(body.to_string())
            .send()
            .await
            .map_err(|e| anyhow!("{e:?}"))?
            .json::<ApiResponse>()
            .await
            .map_err(|e| anyhow!("{e:?}"))
    }
}
