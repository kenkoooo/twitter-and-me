use serde::Deserialize;
use std::collections::HashMap;
use std::env;
use std::fs::File;
use std::io::Read;
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Deserialize, Debug)]
struct Response {
    ids: Vec<i64>,
    next_cursor: i64,
    previous_cursor: i64,
}

#[derive(Deserialize, Debug)]
struct TokenResponse {
    token_type: String,
    access_token: String,
}

#[derive(Deserialize, Debug)]
struct RateLimitStatus {
    resources: RateLimitResources,
}

#[derive(Deserialize, Debug)]
struct RateLimitResources {
    friends: HashMap<String, RateLimitEntry>,
    followers: HashMap<String, RateLimitEntry>,
    application: HashMap<String, RateLimitEntry>,
}

#[derive(Deserialize, Debug)]
struct RateLimitEntry {
    limit: i64,
    remaining: i64,
    reset: i64,
}

#[derive(Deserialize, Debug)]
struct Config {
    key: String,
    secret: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut f = File::open(".twitter.json")?;
    let mut buffer = String::new();
    f.read_to_string(&mut buffer)?;

    let keys: Vec<Config> = serde_json::from_str(&buffer)?;

    for config in keys.iter() {
        match TwitterClient::new(&config.key, &config.secret).await {
            Ok(client) => {
                let response = client.rate_limit_status().await?;
                println!("{:?}", response.resources.friends.get("/friends/ids"));
                println!("{:?}", response.resources.followers.get("/followers/ids"));
            }
            Err(e) => {
                println!("{:?} {:?}", config, e);
            }
        }
    }

    Ok(())
}

struct TwitterClient {
    client: reqwest::Client,
    bearer_token: String,
}

impl TwitterClient {
    async fn new(consumer_key: &str, consumer_secret: &str) -> Result<Self, reqwest::Error> {
        let client = reqwest::ClientBuilder::new().gzip(true).build()?;
        let key = format!("{}:{}", consumer_key, consumer_secret);
        let auth_header = format!("Basic {}", base64::encode(key));
        let response = client
            .post("https://api.twitter.com/oauth2/token")
            .header(
                reqwest::header::CONTENT_TYPE,
                "application/x-www-form-urlencoded;charset=UTF-8",
            )
            .header(reqwest::header::AUTHORIZATION, auth_header)
            .form(&[("grant_type", "client_credentials")])
            .send()
            .await?
            .json::<TokenResponse>()
            .await?;
        Ok(Self {
            bearer_token: response.access_token,
            client,
        })
    }

    async fn rate_limit_status(&self) -> Result<RateLimitStatus, reqwest::Error> {
        self.client
            .get("https://api.twitter.com/1.1/application/rate_limit_status.json")
            .header(
                reqwest::header::AUTHORIZATION,
                format!("Bearer {}", self.bearer_token),
            )
            .query(&[("resources", "friends,followers,application")])
            .send()
            .await?
            .json::<RateLimitStatus>()
            .await
    }

    async fn friends_ids(
        &self,
        screen_name: &str,
        cursor: i64,
    ) -> Result<Response, reqwest::Error> {
        self.client
            .get("https://api.twitter.com/1.1/friends/ids.json")
            .header(
                reqwest::header::AUTHORIZATION,
                format!("Bearer {}", self.bearer_token),
            )
            .query(&[("screen_name", screen_name)])
            .query(&[("cursor", cursor)])
            .query(&[("count", 5000)])
            .send()
            .await?
            .json::<Response>()
            .await
    }
}
