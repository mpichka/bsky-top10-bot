use reqwest::Error as ReqwestError;
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;
use std::fmt;

#[derive(Debug)]
pub enum Error {
    ReqwestError(ReqwestError),
    BskyError(BskyError),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::ReqwestError(err) => write!(f, "Reqwest error: {}", err),
            Error::BskyError(err) => write!(
                f,
                "Bsky error: {{ \"error\": \"{}\", \"message\": \"{}\" }}",
                err.error, err.message
            ),
        }
    }
}

impl std::error::Error for Error {}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct BskyError {
    pub error: BskyErrorCode,
    pub message: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub enum BskyErrorCode {
    InvalidRequest,
    ExpiredToken,
    InvalidToken,
    AccountTakedown,
    AuthFactorTokenRequired,
    #[serde(untagged)]
    Unknown,
}

impl fmt::Display for BskyErrorCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            BskyErrorCode::InvalidRequest => write!(f, "InvalidRequest"),
            BskyErrorCode::ExpiredToken => write!(f, "ExpiredToken"),
            BskyErrorCode::InvalidToken => write!(f, "InvalidToken"),
            BskyErrorCode::AccountTakedown => write!(f, "AccountTakedown"),
            BskyErrorCode::AuthFactorTokenRequired => write!(f, "AuthFactorTokenRequired"),
            BskyErrorCode::Unknown => write!(f, "UnhandledException"),
        }
    }
}

impl std::error::Error for BskyErrorCode {}

#[derive(Serialize)]
pub struct AuthenticationBody {
    pub identifier: String,
    pub password: String,
}

#[derive(Deserialize)]
pub struct AuthenticationResponse {
    #[serde(rename = "accessJwt")]
    pub access_jwt: String,
    #[serde(rename = "refreshJwt")]
    pub refresh_jwt: String,
    pub handle: String,
    pub did: String,
    pub status: Option<AccountStatus>,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum AccountStatus {
    Takendown,
    Suspended,
    Deactivated,
}

impl fmt::Display for AccountStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AccountStatus::Takendown => write!(f, "Takendown"),
            AccountStatus::Suspended => write!(f, "Suspended"),
            AccountStatus::Deactivated => write!(f, "Deactivated"),
        }
    }
}

#[derive(Deserialize)]
pub struct FollowsResponse {
    pub cursor: Option<String>,
    pub follows: Vec<Author>,
}

#[derive(Deserialize)]
pub struct Author {
    pub did: String,
    pub handle: Option<String>,
    #[serde(rename = "displayName")]
    pub display_name: Option<String>,
}

#[derive(Serialize)]
#[serde(rename_all = "snake_case")]
pub enum FeedFilter {
    PostsWithReplies,
    // PostsNoReplies,
    // PostsWithMedia,
    // PostsAndAuthorThreads,
}

impl fmt::Display for FeedFilter {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            FeedFilter::PostsWithReplies => write!(f, "posts_with_replies"),
            // FeedFilter::PostsNoReplies => write!(f, "posts_no_replies"),
            // FeedFilter::PostsWithMedia => write!(f, "posts_with_media"),
            // FeedFilter::PostsAndAuthorThreads => write!(f, "posts_and_author_threads"),
        }
    }
}

#[derive(Serialize)]
pub struct FollowersListOptions {
    pub actor: String,
    pub limit: i8,
    pub cursor: Option<String>,
}

#[derive(Serialize)]
pub struct FeedListOptions {
    pub actor: String,
    pub limit: Option<i8>,
    pub cursor: Option<String>,
    pub filter: Option<FeedFilter>,
}

#[derive(Deserialize)]
pub struct FeedResponse {
    pub feed: Vec<Feed>,
    pub cursor: Option<String>,
}

#[derive(Deserialize)]
pub struct Feed {
    pub post: Post,
    // pub reply: Option<FeedReply>,
    pub reason: Option<Reason>,
}

#[derive(Deserialize)]
pub struct Reason {
    #[serde(rename = "$type")]
    pub reason_type: ReasonType,
    // pub by: Author,
    // #[serde(rename = "indexedAt")]
    // pub indexed_at: Option<String>,
}

#[derive(PartialEq)]
pub enum ReasonType {
    Repost,
    Unknown(String),
}

impl<'de> Deserialize<'de> for ReasonType {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        match s.as_str() {
            "app.bsky.feed.defs#reasonRepost" => Ok(ReasonType::Repost),
            other => Ok(ReasonType::Unknown(other.to_string())),
        }
    }
}

#[derive(Deserialize)]
pub struct Post {
    pub uri: String,
    pub cid: Option<String>,
    // pub author: Option<Author>,
    pub record: Option<Record>,
    #[serde(rename = "replyCount")]
    pub reply_count: Option<i32>,
    #[serde(rename = "repostCount")]
    pub repost_count: Option<i32>,
    #[serde(rename = "likeCount")]
    pub like_count: Option<i32>,
    #[serde(rename = "quoteCount")]
    pub quote_count: Option<i32>,
    // #[serde(rename = "indexedAt")]
    // pub indexed_at: Option<String>,
    // #[serde(rename = "notFound")]
    // pub not_found: Option<bool>,
    // pub blocked: Option<bool>,
}

#[derive(Deserialize)]
pub struct Record {
    // pub text: String,
    #[serde(rename = "createdAt")]
    pub created_at: String,
    // pub langs: Option<Vec<String>>,
    // pub tags: Option<Vec<String>>,
}

#[derive(Serialize)]
pub struct NewPost {
    pub repo: String,
    pub collection: RecordType,
    pub record: NewRecord,
    pub lang: Vec<String>,
    pub validate: bool,
}

#[skip_serializing_none]
#[derive(Serialize)]
pub struct NewRecord {
    #[serde(rename = "$type")]
    pub record_type: RecordType,
    pub text: String,
    #[serde(rename = "createdAt")]
    pub created_at: String,
    pub facets: Option<Vec<Facet>>,
    pub reply: Option<Reply>,
    pub embed: Option<Embed>,
}

#[derive(Serialize)]
pub struct Reply {
    pub root: PostRef,
    pub parent: PostRef,
}

#[derive(Serialize, Deserialize)]
pub struct PostRef {
    pub uri: String,
    pub cid: String,
}

#[derive(Serialize)]
pub enum RecordType {
    #[serde(rename = "app.bsky.feed.post")]
    Post,
}

#[derive(Serialize)]
pub struct Facet {
    pub index: Index,
    pub features: Vec<Feature>,
}

#[skip_serializing_none]
#[derive(Serialize)]
pub struct Feature {
    #[serde(rename = "$type")]
    pub feature_type: FeatureType,
    pub did: Option<String>,
    pub uri: Option<String>,
    pub tag: Option<String>,
}

#[derive(Serialize)]
pub struct Index {
    #[serde(rename = "byteStart")]
    pub byte_start: u64,
    #[serde(rename = "byteEnd")]
    pub byte_end: u64,
}

#[derive(Serialize)]
pub enum FeatureType {
    #[serde(rename = "app.bsky.richtext.facet#mention")]
    Mention,
    #[serde(rename = "app.bsky.richtext.facet#link")]
    Link,
    #[serde(rename = "app.bsky.richtext.facet#tag")]
    Tag,
}

#[derive(Serialize)]
pub struct Embed {
    #[serde(rename = "$type")]
    pub embed_type: EmbedType,
    pub record: PostRef,
}

#[derive(Serialize)]
pub enum EmbedType {
    #[serde(rename = "app.bsky.embed.record")]
    Record,
}
