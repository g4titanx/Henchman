//! API client for interacting with the Hyperbolic LLM service
use crate::config::HyperbolicConfig;

use anyhow::{anyhow, Result};
use reqwest::Client;

use self::api_types::ApiResponse;
mod api_types;

/// Client for making requests to the Hyperbolic API service.
/// Handles authentication and text generation requests.
pub struct HyperbolicClient {
    /// Base URL for the Hyperbolic API
    base_url: String,
    /// API key for authentication
    hyperbolic_api_key: String,
    /// HTTP client for making requests
    client: Client,
}

impl HyperbolicClient {
    /// Creates a new HyperbolicClient instance
    pub fn new(hyperbolic_api_key: String, base_url: String) -> Self {
        let client = Client::new();
        Self {
            base_url,
            hyperbolic_api_key,
            client,
        }
    }

    /// Generates text using the Hyperbolic LLM service
    pub async fn generate_text(
        &self,
        context: &str,
        prompt: &str,
        config: &HyperbolicConfig,
    ) -> Result<ApiResponse> {
        let url = format!("{}/chat/completions", self.base_url);

        let body = serde_json::json!({
            "messages": [
                {
                    "role": "system",
                    "content": context,
                },
                {
                    "role": "user",
                    "content": prompt
                }
            ],
            "model": config.model,
            "max_tokens": config.max_tokens,
            "temperature": config.temperature,
            "top_p": config.top_p,
            "top_k": config.top_k,
            "stream": false,
        });

        self.client
            .post(&url)
            .header("Content-Type", "application/json")
            .header(
                "Authorization",
                format!("Bearer {}", self.hyperbolic_api_key),
            )
            .body(body.to_string())
            .send()
            .await
            .map_err(|e| anyhow!("{e:?}"))?
            .json::<ApiResponse>()
            .await
            .map_err(|e| anyhow!("{e:?}"))
    }
}

#[cfg(test)]
mod tests {
    use crate::config::HyperbolicConfig;
    use crate::hyperbolic::HyperbolicClient;

    #[ignore]
    #[tokio::test]
    async fn test_generate_text() {
        let base_url = "https://api.hyperbolic.xyz/v1".to_string();
        let hyperbolic_api_key = "".to_string();
        let client = HyperbolicClient::new(hyperbolic_api_key, base_url);

        let config = HyperbolicConfig {
            model: "meta-llama/Meta-Llama-3.1-70B-Instruct".to_string(),
            max_tokens: 512,
            temperature: 1.0,
            top_p: 0.95,
            top_k: 40,
        };

        let res = client
            .generate_text(
                "hey shitalik, when does ethereum go to zero?",
                "write a witty response to this tweet",
                &config,
            )
            .await
            .unwrap();

        println!("{res:?}");
    }
}
