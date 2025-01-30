use serde::Deserialize;

/// Response from OpenAI's embedding API containing vector representations of the input text.
/// The embeddings can be used for semantic similarity search and other NLP tasks.
#[derive(Deserialize, Debug)]
pub struct ApiResponse {
    /// Type identifier for the response object
    pub object: String,
    /// List of embedding vectors generated from the input text
    pub data: Vec<EmbeddingData>,
    /// Identifier of the model used to generate embeddings
    pub model: String,
    /// Token usage statistics for the request
    pub usage: Usage,
}

/// Contains the embedding vector and metadata for a single piece of text.
#[derive(Deserialize, Debug)]
pub struct EmbeddingData {
    /// Type identifier for this embedding
    pub object: String,
    /// Position of this embedding in the request (if multiple inputs were provided)
    pub index: usize,
    /// Vector representation of the input text
    /// Each component is a float value, and the vector dimensions are consistent
    /// for a given model
    pub embedding: Vec<f32>,
}

/// Tracks token usage for the embedding request for billing and monitoring.
#[derive(Deserialize, Debug)]
pub struct Usage {
    /// Number of tokens in the input text
    pub prompt_tokens: usize,
    /// Total tokens processed (same as prompt_tokens for embeddings)
    pub total_tokens: usize,
}
