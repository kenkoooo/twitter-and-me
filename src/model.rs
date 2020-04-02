use serde::Deserialize;
use std::collections::HashMap;

#[derive(Deserialize, Debug)]
pub struct Ids {
    pub ids: Vec<i64>,
    pub next_cursor: i64,
    pub previous_cursor: i64,
}

#[derive(Deserialize, Debug)]
pub struct TokenResponse {
    pub token_type: String,
    pub access_token: String,
}

#[derive(Deserialize, Debug)]
pub struct RateLimitStatus {
    pub resources: RateLimitResources,
}

impl RateLimitStatus {
    pub fn friends_ids(&self) -> Option<i64> {
        self.resources
            .friends
            .get("/friends/ids")
            .map(|e| e.remaining)
    }
    pub fn followers_ids(&self) -> Option<i64> {
        self.resources
            .followers
            .get("/followers/ids")
            .map(|e| e.remaining)
    }
    pub fn users_lookup(&self) -> Option<i64> {
        self.resources
            .users
            .get("/users/lookup")
            .map(|e| e.remaining)
    }
}

#[derive(Deserialize, Debug)]
pub struct RateLimitResources {
    pub friends: HashMap<String, RateLimitEntry>,
    pub followers: HashMap<String, RateLimitEntry>,
    pub application: HashMap<String, RateLimitEntry>,
    pub users: HashMap<String, RateLimitEntry>,
}

#[derive(Deserialize, Debug)]
pub struct RateLimitEntry {
    pub limit: i64,
    pub remaining: i64,
    pub reset: i64,
}

#[derive(Deserialize, Debug)]
pub struct User {
    pub id: i64,
    pub name: String,
    pub screen_name: String,
    pub description: String,
    pub protected: bool,
    pub followers_count: i64,
    pub friends_count: i64,
    pub favourites_count: i64,
    pub statuses_count: i64,
    pub created_at: String,
    pub profile_image_url_https: String,
}
