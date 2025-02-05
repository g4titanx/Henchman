use crate::twitter::api_types::TimelineTweet;
use std::path::PathBuf;

use anyhow::{anyhow, Context, Result};
use qdrant_client::{
    qdrant::{
        CreateCollectionBuilder, Distance, PointStruct, ScalarQuantizationBuilder,
        SearchParamsBuilder, SearchPointsBuilder, UpsertPointsBuilder, VectorParamsBuilder,
    },
    Payload, Qdrant,
};
use rocksdb::{IteratorMode, Options, DB};
use uuid::Uuid;

use self::types::{Embedding, Memory, MemoryData};

pub mod types;

/// Column family for storing processed tweet IDs
const TWEET_IDS: &str = "tweet_ids";
/// Column family for storing memory data
const MEMORY_DATA: &str = "memory-data";
/// Column family for storing user IDs
const USER_ID: &str = "user-id";
/// Column family for buffering tweets
const TWEET_BUFFER: &str = "tweet-buffer";

/// Database provides a hybrid storage system combining vector-based similarity search (Qdrant)
/// with traditional key-value storage (RocksDB). This system manages the AI agent's memory,
/// tweet history, and user interactions.
pub struct Database {
    /// Client for the Qdrant vector database used for similarity search
    pub vec_db_client: Qdrant,
    /// RocksDB instance for key-value storage
    pub kv_db: DB,
}

impl Database {
    /// Creates a new Database instance by connecting to both Qdrant and RocksDB.
    pub fn new(vector_db_url: &str, kv_db_path: PathBuf) -> Result<Self> {
        let vec_db_client = Qdrant::from_url(vector_db_url).build()?;

        let mut db_options = Options::default();
        db_options.create_if_missing(true);
        db_options.create_missing_column_families(true);

        let cf = vec![TWEET_IDS, MEMORY_DATA, USER_ID];
        let kv_db = DB::open_cf(&db_options, kv_db_path, cf)?;

        Ok(Self {
            vec_db_client,
            kv_db,
        })
    }

    /// Creates a new vector collection in Qdrant if it doesn't exist.
    ///
    /// # Arguments
    /// * `collection_name` - Name of the collection to create
    /// * `vector_dim` - Dimension of vectors to be stored (e.g., 1536 for OpenAI embeddings)
    pub async fn create_collection(&self, collection_name: &str, vector_dim: u64) -> Result<()> {
        if self
            .vec_db_client
            .collection_exists(collection_name)
            .await?
        {
            Ok(())
        } else {
            self.vec_db_client
                .create_collection(
                    CreateCollectionBuilder::new(collection_name)
                        .vectors_config(VectorParamsBuilder::new(vector_dim, Distance::Cosine))
                        .quantization_config(ScalarQuantizationBuilder::default()),
                )
                .await?;
            Ok(())
        }
    }

    /// Atomically inserts or updates memories in both vector and key-value stores.
    pub async fn upsert_memories(
        &self,
        collection_name: &str,
        memories: Vec<Memory>,
    ) -> Result<()> {
        // First, try to insert all memory data
        let mut inserted_ids = Vec::new();
        for memory in &memories {
            match self.insert_memory_data(memory.data.clone()) {
                Ok(_) => inserted_ids.push(memory.data.id),
                Err(e) => {
                    for id in inserted_ids {
                        let cf = self
                            .kv_db
                            .cf_handle(MEMORY_DATA)
                            .expect("failed to get memory data cf handle");
                        let _ = self.kv_db.delete_cf(&cf, id.to_le_bytes());
                    }
                    return Err(e);
                }
            }
        }

        // If all memory data is inserted, try to upsert points
        let points: Vec<PointStruct> = memories
            .iter()
            .map(|m| {
                let payload: Payload = serde_json::json!(
                    {
                        "id": m.data.id
                    }
                )
                .try_into()
                .unwrap();
                let id_hash = fasthash::spooky::hash128(m.data.id.to_le_bytes());
                let id = Uuid::from_bytes(id_hash.to_le_bytes());
                PointStruct::new(id.to_string(), m.embedding.data.clone(), payload)
            })
            .collect();

        match self
            .vec_db_client
            .upsert_points(UpsertPointsBuilder::new(collection_name, points))
            .await
        {
            Ok(_) => Ok(()),
            Err(e) => {
                // Roll back memory data inserts on vector db failure
                for id in inserted_ids {
                    let cf = self
                        .kv_db
                        .cf_handle(MEMORY_DATA)
                        .expect("Failed to get memory data cf handle");
                    let _ = self.kv_db.delete_cf(&cf, id.to_le_bytes());
                }
                Err(anyhow!("Failed to upsert vectors: {}", e))
            }
        }
    }

    /// Retrieves the k most similar memories to a given embedding using cosine similarity
    pub async fn get_k_most_similar_memories(
        &self,
        collection_name: &str,
        embedding: Embedding,
        k: u64,
    ) -> Result<Vec<MemoryData>> {
        let search_result = self
            .vec_db_client
            .search_points(
                SearchPointsBuilder::new(collection_name, embedding.data, k)
                    //.filter(Filter::all([Condition::matches("bar", 12)]))
                    .with_payload(true)
                    .params(SearchParamsBuilder::default().exact(true)),
            )
            .await
            .unwrap();

        let ids = search_result
            .result
            .iter()
            .filter_map(|r| {
                if let Some(content) = r.payload.get("id") {
                    let id = content.as_integer().unwrap() as u128;
                    Some(id)
                } else {
                    None
                }
            })
            .collect::<Vec<u128>>();
        let mut memories = Vec::with_capacity(ids.len());
        for id in ids {
            let memory = self.get_memory(id)?;
            memories.push(memory);
        }

        Ok(memories)
    }

    /// Records a processed tweet ID to prevent duplicate processing
    pub fn insert_tweet_id(&self, tweet_id: &str) -> Result<()> {
        let tweed_id_cf = self
            .kv_db
            .cf_handle(TWEET_IDS)
            .expect("failed to get tweet id cf handle");
        self.kv_db
            .put_cf(&tweed_id_cf, tweet_id.as_bytes(), b"0")
            .map_err(|e| anyhow!("{e:?}"))
    }

    /// Checks if a user ID exists in the database
    pub fn user_id_exists(&self, user_id: &str) -> Result<bool> {
        let user_id_cf = self
            .kv_db
            .cf_handle(USER_ID)
            .expect("failed to get user id cf handle");
        self.kv_db
            .get_cf(&user_id_cf, user_id.as_bytes())
            .map(|v| v.is_some())
            .map_err(|e| anyhow!("{e:?}"))
    }

    /// Records a user ID, typically used for tracking followed users
    pub fn insert_user_id(&self, user_id: &str) -> Result<()> {
        let user_id_cf = self
            .kv_db
            .cf_handle(USER_ID)
            .expect("failed to get user id cf handle");
        self.kv_db
            .put_cf(&user_id_cf, user_id.as_bytes(), b"0")
            .map_err(|e| anyhow!("{e:?}"))
    }

    /// Checks if a tweet has been previously processed
    pub fn tweet_id_exists(&self, tweet_id: &str) -> Result<bool> {
        let tweed_id_cf = self
            .kv_db
            .cf_handle(TWEET_IDS)
            .expect("failed to get tweet id cf handle");
        self.kv_db
            .get_cf(&tweed_id_cf, tweet_id.as_bytes())
            .map(|v| v.is_some())
            .map_err(|e| anyhow!("{e:?}"))
    }

    /// Inserts memory data into the key-value store
    pub fn insert_memory_data(&self, data: MemoryData) -> Result<()> {
        let cf = self
            .kv_db
            .cf_handle(MEMORY_DATA)
            .expect("failed to get memory data cf handle");
        let data_bytes = bincode::serialize(&data)?;
        self.kv_db
            .put_cf(&cf, data.id.to_le_bytes(), data_bytes)
            .map_err(|e| anyhow!("{e:?}"))
    }

    /// Retrieves memory data by ID
    fn get_memory(&self, id: u128) -> Result<MemoryData> {
        let cf = self
            .kv_db
            .cf_handle(MEMORY_DATA)
            .expect("failed to get memory data cf handle");

        let data = self
            .kv_db
            .get_cf(&cf, id.to_le_bytes())?
            .context("failed to get memory")?;
        bincode::deserialize::<MemoryData>(&data).map_err(|e| anyhow!("{e:?}"))
    }

    /// Retrieves the most recent memories, quite useful for maintaining context
    pub fn get_recent_memories(&self, max_num: usize) -> Result<Vec<MemoryData>> {
        let sent_tweed_cf = self
            .kv_db
            .cf_handle(MEMORY_DATA)
            .expect("failed to get sent memory data cf handle");
        let iter = self.kv_db.iterator_cf(sent_tweed_cf, IteratorMode::End);
        let mut memories = Vec::with_capacity(max_num);
        for (_key, val) in iter.flatten() {
            if memories.len() >= max_num {
                break;
            }
            if let Ok(memory) = bincode::deserialize::<MemoryData>(&val) {
                memories.push(memory);
            }
        }

        Ok(memories)
    }

    /// Stores a tweet in the buffer for later processing
    pub fn store_buffered_tweet(&self, tweet: &TimelineTweet) -> Result<()> {
        let cf = self
            .kv_db
            .cf_handle(TWEET_BUFFER)
            .expect("failed to get tweet buffer cf handle");
        let tweet_bytes = bincode::serialize(tweet)?;
        self.kv_db
            .put_cf(&cf, tweet.id.as_bytes(), tweet_bytes)
            .map_err(|e| anyhow!("{e:?}"))
    }

    /// Retrieves buffered tweets up to the specified limit
    pub fn get_buffered_tweets(&self, limit: usize) -> Result<Vec<TimelineTweet>> {
        let cf = self
            .kv_db
            .cf_handle(TWEET_BUFFER)
            .expect("failed to get tweet buffer cf handle");
        let iter = self.kv_db.iterator_cf(cf, IteratorMode::Start);
        let mut tweets = Vec::with_capacity(limit);

        for item in iter {
            if tweets.len() >= limit {
                break;
            }
            let (_, value) = item.map_err(|e| anyhow!("{e:?}"))?;
            let tweet: TimelineTweet =
                bincode::deserialize(&value).map_err(|e| anyhow!("{e:?}"))?;
            tweets.push(tweet);
        }

        Ok(tweets)
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use crate::db::{
        types::{Embedding, Memory, MemoryData},
        Database,
    };

    #[ignore]
    #[tokio::test]
    async fn test_vector_db() {
        let db =
            Database::new("http://localhost:6334", PathBuf::from("/tmp/rocksdb_test")).unwrap();

        let table = "test6";
        db.create_collection(table, 3).await.unwrap();

        let mem1 = Memory {
            data: MemoryData {
                id: 0,
                score: 1,
                content: String::from("hello"),
            },
            embedding: Embedding::new(0, vec![1.2, 2.2, 3.2]),
        };
        let mem2 = Memory {
            data: MemoryData {
                id: 1,
                score: 2,
                content: String::from("goodbye"),
            },
            embedding: Embedding::new(1, vec![1.1, 2.1, 3.1]),
        };
        let mem3 = Memory {
            data: MemoryData {
                id: 2,
                score: 3,
                content: String::from("foo"),
            },
            embedding: Embedding::new(2, vec![1.3, 2.3, 3.3]),
        };
        let mem4 = Memory {
            data: MemoryData {
                id: 3,
                score: 4,
                content: String::from("bar"),
            },
            embedding: Embedding::new(3, vec![1.4, 2.4, 3.4]),
        };

        let memories = vec![mem1, mem2, mem3, mem4];

        db.upsert_memories(table, memories).await.unwrap();

        let res = db
            .get_k_most_similar_memories(table, Embedding::new(0, vec![1., 2., 3.]), 2)
            .await
            .unwrap();

        for r in &res {
            println!("{r:?}");
        }
    }

    #[ignore]
    #[tokio::test]
    async fn test_insert_sent_tweets() {
        let db =
            Database::new("http://localhost:6334", PathBuf::from("/tmp/rocksdb_test")).unwrap();

        db.insert_memory_data(MemoryData {
            id: 1,
            score: 1,
            content: "y".to_string(),
        })
        .unwrap();
        db.insert_memory_data(MemoryData {
            id: 2,
            score: 1,
            content: "x".to_string(),
        })
        .unwrap();
        db.insert_memory_data(MemoryData {
            id: 3,
            score: 3,
            content: "foo".to_string(),
        })
        .unwrap();
        db.insert_memory_data(MemoryData {
            id: 4,
            score: 1,
            content: "bar".to_string(),
        })
        .unwrap();

        let tweets = db.get_recent_memories(2).unwrap();
        assert_eq!(tweets[0].content, "bar");
        assert_eq!(tweets[1].content, "foo");
    }
}
