// use tempfile::tempdir;
// use anyhow::Result;

use crate::{
    // db::{Database, types::{Memory, MemoryData, Embedding}},
    prompts::Prompts,
};

// /// Tests for the Database component
// mod database_tests {
//     use super::*;
    
//     /// Helper function to create a temporary test database
//     async fn setup_test_db() -> Result<(Database, tempfile::TempDir)> {
//         let temp_dir = tempdir()?;
//         let db_path = temp_dir.path().join("test_db");
//         let db = Database::new("http://localhost:6334", db_path)?;
        
//         // Create required collections
//         db.create_collection("test_memories", 1536).await?;
        
//         Ok((db, temp_dir))
//     }

//     #[tokio::test]
//     async fn test_memory_storage_and_retrieval() -> Result<()> {
//         let (db, _temp) = setup_test_db().await?;

//         // Create test memories
//         let memories = vec![
//             Memory {
//                 data: MemoryData {
//                     id: 1,
//                     score: 8,
//                     content: "Ethereum hit new ATH!".to_string(),
//                 },
//                 embedding: Embedding::new(1, vec![0.5; 1536]),
//             },
//             Memory {
//                 data: MemoryData {
//                     id: 2,
//                     score: 5,
//                     content: "Just another day in crypto".to_string(),
//                 },
//                 embedding: Embedding::new(2, vec![0.3; 1536]),
//             },
//         ];

//         // Store memories
//         db.upsert_memories("test_memories", memories).await?;

//         // Test retrieval 
//         let query_embedding = Embedding::new(0, vec![0.5; 1536]);
//         let results = db.get_k_most_similar_memories("test_memories", query_embedding, 2).await?;

//         assert_eq!(results.len(), 2);
//         assert!(results.iter().any(|m| m.content.contains("ATH")));

//         Ok(())
//     }

//     #[tokio::test]
//     async fn test_tweet_id_tracking() -> Result<()> {
//         let (db, _temp) = setup_test_db().await?;
        
//         let tweet_id = "123456789";
        
//         // Initially tweet should not exist
//         assert!(!db.tweet_id_exists(tweet_id)?);
        
//         // Insert tweet
//         db.insert_tweet_id(tweet_id)?;
        
//         // Now tweet should exist
//         assert!(db.tweet_id_exists(tweet_id)?);
        
//         Ok(())
//     }

//     #[tokio::test]
//     async fn test_user_id_tracking() -> Result<()> {
//         let (db, _temp) = setup_test_db().await?;
        
//         let user_id = "user_123";
        
//         // Initially user should not exist
//         assert!(!db.user_id_exists(user_id)?);
        
//         // Insert user
//         db.insert_user_id(user_id)?;
        
//         // Now user should exist
//         assert!(db.user_id_exists(user_id)?);
        
//         Ok(())
//     }
    
//     #[tokio::test]
//     async fn test_recent_memories() -> Result<()> {
//         let (db, _temp) = setup_test_db().await?;

//         // Insert test memories in order
//         for i in 1..=5 {
//             db.insert_memory_data(MemoryData {
//                 id: i,
//                 score: 5,
//                 content: format!("Memory {}", i),
//             })?;
//         }

//         // Test retrieval with limit
//         let recent = db.get_recent_memories(3)?;
//         assert_eq!(recent.len(), 3);
//         assert_eq!(recent[0].content, "Memory 5"); // Most recent first
//         assert_eq!(recent[2].content, "Memory 3");

//         Ok(())
//     }
// }

/// Tests for the Prompts component
mod prompts_tests {
    use super::*;

    #[test]
    fn test_mentions_prompt_generation() {
        let prompts = Prompts::load();
        let tweets = vec![
            "Tweet 1 content".to_string(),
            "Tweet 2 content".to_string(),
        ];

        let prompt = prompts.get_mentions_prompt(tweets);
        assert!(prompt.contains("Tweet 1 content"));
        assert!(prompt.contains("Tweet 2 content"));
    }

    #[test]
    fn test_tweet_prompt_generation() {
        let prompts = Prompts::load();
        let prompt = prompts.get_tweet_prompt(
            "Short term memory".to_string(),
            vec!["Long memory 1".to_string()],
            vec!["Recent post 1".to_string()],
            vec!["Context 1".to_string()],
        );

        assert!(prompt.contains("Short term memory"));
        assert!(prompt.contains("Long memory 1"));
        assert!(prompt.contains("Recent post 1"));
        assert!(prompt.contains("Context 1"));
    }

    #[test]
    fn test_wallet_decision_prompt() {
        let prompts = Prompts::load();
        let prompt = prompts.get_wallet_decision_prompt(
            vec!["Post with wallet".to_string()],
            vec!["0x123...".to_string()],
            "1.5".to_string(),
        );

        assert!(prompt.contains("Post with wallet"));
        assert!(prompt.contains("0x123..."));
        assert!(prompt.contains("1.5"));
    }
}

/// Tests for pipeline timing configuration
mod pipeline_tests {
    use crate::pipeline::PipelineConfig;
    use std::time::Duration;

    #[test]
    fn test_pipeline_timing_bounds() {
        let config = PipelineConfig::default();
        
        // Test 100 random timing generations
        for _ in 0..100 {
            // Test scroll sleep timing
            let scroll_sleep = config.get_scroll_sleep_time();
            assert!(scroll_sleep >= Duration::from_secs(0));
            assert!(scroll_sleep <= Duration::from_secs(1800));
            
            // Test scroll duration timing
            let scroll_duration = config.get_scroll_duration_time();
            assert!(scroll_duration >= Duration::from_secs(900));
            assert!(scroll_duration <= Duration::from_secs(1200));
            
            // Test run sleep timing
            let run_sleep = config.get_run_sleep_time();
            assert!(run_sleep >= Duration::from_secs(30));
            assert!(run_sleep <= Duration::from_secs(180));
        }
    }
}

/// Tests for embedding calculations
mod embedding_tests {
    use crate::db::types::Embedding;

    #[test]
    fn test_cosine_similarity() {
        // Test identical vectors
        let emb1 = Embedding::new(1, vec![1.0, 0.0, 0.0]);
        let emb2 = Embedding::new(2, vec![1.0, 0.0, 0.0]);
        assert_eq!(emb1.cosine_similarity(&emb2), 1.0);

        // Test orthogonal vectors
        let emb3 = Embedding::new(3, vec![1.0, 0.0, 0.0]);
        let emb4 = Embedding::new(4, vec![0.0, 1.0, 0.0]);
        assert_eq!(emb3.cosine_similarity(&emb4), 0.0);

        // Test opposite vectors
        let emb5 = Embedding::new(5, vec![1.0, 0.0, 0.0]);
        let emb6 = Embedding::new(6, vec![-1.0, 0.0, 0.0]);
        assert_eq!(emb5.cosine_similarity(&emb6), -1.0);
    }

    #[test]
    fn test_l2_norm() {
        let emb = Embedding::new(1, vec![3.0, 4.0]);
        assert_eq!(emb.l2_norm(), 5.0); // 3-4-5 triangle
    }
}