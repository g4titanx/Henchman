use serde::{Deserialize, Serialize};
use std::fmt::Display;

/// Generic wrapper for Twitter API responses.
#[derive(Deserialize, Debug)]
pub struct ApiResponse<T> {
    /// The actual response data
    pub data: T,
}

/// Response structure for tweet mentions.
/// Contains a list of tweets mentioning the user along with user information.
#[derive(Deserialize, Debug)]
pub struct MentionsResponse {
    /// Vector of tweets mentioning the user
    pub data: Vec<Tweet>,
    /// Additional user information for tweet authors
    pub includes: IncludesUsers,
    /// Pagination and result metadata
    pub meta: Meta,
}

/// Metadata for paginated API responses.
#[derive(Deserialize, Debug)]
pub struct Meta {
    /// ID of the most recent tweet in the response
    pub newest_id: String,
    /// ID of the oldest tweet in the response
    pub oldest_id: String,
    /// Number of tweets returned in this response
    pub result_count: u32,
}

/// Represents a single tweet with its metadata.
#[derive(Deserialize, Debug)]
pub struct Tweet {
    /// Unique identifier for the tweet
    pub id: String,
    /// ID of the user who created the tweet
    pub author_id: String,
    /// The actual text content of the tweet
    pub text: String,
    /// IDs of previous versions if the tweet was edited
    pub edit_history_tweet_ids: Vec<String>,
    /// Timestamp when the tweet was created
    pub created_at: String,
    /// Optional username of the tweet author
    pub username: Option<String>,
}

/// Response structure for retrieving a user's tweets.
#[derive(Deserialize, Debug)]
pub struct TweetsResponse {
    /// Vector of tweets from the user
    pub data: Vec<Tweet>,
    /// Pagination and result metadata
    pub meta: Meta,
}

/// Represents a tweet sent by the bot.
/// Used for post responses from the Twitter API.
#[derive(Serialize, Deserialize, Debug)]
pub struct SentTweet {
    /// Text content of the sent tweet
    pub text: String,
    /// Unique identifier assigned to the tweet
    pub id: String,
    /// Edit history IDs (initially just contains the original tweet ID)
    pub edit_history_tweet_ids: Vec<String>,
}

/// Represents a Twitter user's basic information.
#[derive(Deserialize, Debug)]
pub struct User {
    /// Unique identifier for the user
    pub id: String,
    /// Display name of the user
    pub name: String,
    /// Username/handle of the user (without @ symbol)
    pub username: String,
}

/// Response structure for timeline requests.
/// Contains tweets from the user's timeline along with user information.
#[derive(Debug, Deserialize)]
pub struct TimelineResponse {
    /// Vector of tweets from the timeline
    pub data: Vec<TimelineTweet>,
    /// Additional user information for tweet authors
    pub includes: IncludesUsers,
    /// Pagination and metadata for timeline results
    pub meta: TimelineMeta,
}

/// Represents a tweet from a user's timeline.
/// Similar to Tweet but with additional fields specific to timeline views.
#[derive(Debug, Deserialize, Clone, Serialize)]
pub struct TimelineTweet {
    /// IDs of previous versions if the tweet was edited
    pub edit_history_tweet_ids: Vec<String>,
    /// Optional article information if the tweet contains a link
    pub article: Option<Article>,
    /// The actual text content of the tweet
    pub text: String,
    /// ID of the user who created the tweet
    pub author_id: String,
    /// Unique identifier for the tweet
    pub id: String,
    /// Optional username of the tweet author
    pub username: Option<String>,
}

/// Article metadata for tweets containing links.
#[derive(Debug, Deserialize, Clone, Serialize)]
pub struct Article {
    /// Title of the linked article
    pub title: String,
}

/// Collection of user information included in API responses.
#[derive(Debug, Deserialize)]
pub struct IncludesUsers {
    /// Vector of user information for relevant tweet authors
    pub users: Vec<User>,
}

/// Extended metadata for timeline responses.
#[derive(Debug, Deserialize)]
pub struct TimelineMeta {
    /// Token for retrieving the next page of results
    pub next_token: String,
    /// Number of tweets in this response
    pub result_count: u32,
    /// ID of the most recent tweet in the response
    pub newest_id: String,
    /// ID of the oldest tweet in the response
    pub oldest_id: String,
}

/// Response data for follow requests.
#[derive(Debug, Deserialize)]
pub struct FollowData {
    /// Whether the follow relationship exists
    pub following: bool,
    /// Whether the follow request is pending approval
    pub pending_follow: bool,
}

impl Display for Tweet {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(username) = &self.username {
            write!(f, "A tweet on my timeline from @{username}: {}", self.text)
        } else {
            write!(f, "New tweet on my timeline: {}", self.text)
        }
    }
}

impl Display for TimelineTweet {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(username) = &self.username {
            write!(f, "A tweet on my timeline from @{username}: {}", self.text)
        } else {
            write!(f, "New tweet on my timeline: {}", self.text)
        }
    }
}

impl Display for SentTweet {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "A tweet from us: {}", self.text)
    }
}
