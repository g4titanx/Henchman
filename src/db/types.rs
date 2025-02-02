use std::fmt::Display;

use serde::{Deserialize, Serialize};

/// Memory combines vector embeddings with associated metadata
/// This structure is used to store and retrieve AI agent memories
/// using both vector similarity search and traditional key-value lookups
#[derive(Debug)]
pub struct Memory {
    /// Vector embedding for similarity search
    pub embedding: Embedding,
    /// Associated metadata and content
    pub data: MemoryData,
}

/// Embedding represents a vector embedding with associated operations for similarity calculations
/// Used primarily for semantic search of memories using cosine similarity
#[derive(Debug)]
pub struct Embedding {
    /// Unique identifier for the embedding
    pub id: u128,
    /// Vector components of the embedding
    pub data: Vec<f32>,
}

/// MemoryData contains the actual content and metadata of a memory
/// This structure is serialized and stored in the key-value database
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct MemoryData {
    /// Unique identifier matching the associated embedding
    pub id: u128,
    /// Significance score of the memory (higher = more significant)
    pub score: u16,
    /// Actual content of the memory (e.g., tweet text)
    pub content: String,
}

impl Embedding {
    /// Creates a new Embedding instance
    ///
    /// # Example
    /// ```no_run
    /// let embedding = Embedding::new(1, vec![0.1, 0.2, 0.3]);
    /// ```
    pub fn new(id: u128, data: Vec<f32>) -> Self {
        Self { id, data }
    }

    /// Calculates the cosine similarity between this embedding and another
    /// Returns a value between -1 and 1, where 1 indicates maximum similarity
    ///
    /// # Example
    /// ```
    /// let emb1 = Embedding::new(1, vec![1.0, 0.0]);
    /// let emb2 = Embedding::new(2, vec![0.0, 1.0]);
    /// let similarity = emb1.cosine_similarity(&emb2); // Returns 0.0 (perpendicular vectors)
    /// ```
    pub fn cosine_similarity(&self, rhs: &Embedding) -> f32 {
        let dot = self.dot(rhs);
        let norm1 = self.l2_norm();
        let norm2 = rhs.l2_norm();
        
        if norm1 == 0.0 || norm2 == 0.0 {
            return 0.0; // Handle zero vectors
        }
        
        dot / (norm1 * norm2)
    }

    /// Calculates the dot product between this embedding and another
    /// Assumes both embeddings have the same dimensions
    ///
    /// # Arguments
    /// * `rhs` - The other embedding to calculate dot product with
    ///
    /// # Returns
    /// * `f32` - Dot product value
    ///
    /// # Note
    /// The embeddings returned from the OpenAI API always have the same dimensions,
    /// so we don't perform dimension checking.
    pub fn dot(&self, rhs: &Embedding) -> f32 {
        self.data
            .iter()
            .zip(rhs.data.iter())
            .map(|(x, y)| x * y)
            .sum::<f32>()
    }

    /// Calculates the L2 (Euclidean) norm of the embedding vector.
    /// Returns an `f32` - L2 norm value
    pub fn l2_norm(&self) -> f32 {
        self.data
            .iter()
            .map(|x| x * x)
            .sum::<f32>()
            .sqrt()
    }
}

impl Display for MemoryData {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "A tweet from us: {}", self.content)
    }
}
