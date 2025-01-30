use serde::Deserialize;

/// Response from the Hyperbolic API for text generation requests
/// Contains the generated text along with metadata about the request
#[derive(Deserialize, Debug)]
pub struct ApiResponse {
    /// Unique identifier for this response
    pub id: String,
    /// Object type identifier from the API
    pub object: String,
    /// Unix timestamp of when the response was created
    pub created: u64,
    /// Name/identifier of the model used for generation
    pub model: String,
    /// List of generated text choices/completions
    pub choices: Vec<Choice>,
    /// Token usage statistics for the request
    pub usage: Usage,
}

/// Represents a single generated completion/choice from the model.
#[derive(Deserialize, Debug)]
pub struct Choice {
    /// Index of this choice in the response
    pub index: usize,
    /// The generated message content and metadata
    pub message: Message,
    /// Reason why the model stopped generating
    pub finish_reason: String,
    /// Optional log probabilities for the generated tokens
    pub logprobs: Option<Vec<f64>>,
}

/// Contains the role and content of a message in the conversation.
#[derive(Deserialize, Debug)]
pub struct Message {
    /// Role of the message sender (e.g., "system", "user", "assistant")
    pub role: String,
    /// Actual text content of the message
    pub content: String,
}

/// Tracks token usage for the request for billing and monitoring.
#[derive(Deserialize, Debug)]
pub struct Usage {
    /// Number of tokens in the prompt
    pub prompt_tokens: usize,
    /// Total tokens used (prompt + completion)
    pub total_tokens: usize,
    /// Number of tokens in the completion
    pub completion_tokens: usize,
}
