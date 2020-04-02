use crate::model::{Ids, RateLimitStatus, TokenResponse, User};

pub struct TwitterClient {
    client: reqwest::Client,
    bearer_token: String,
}

impl TwitterClient {
    pub async fn new(consumer_key: &str, consumer_secret: &str) -> Result<Self, reqwest::Error> {
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

    pub async fn rate_limit_status(&self) -> Result<RateLimitStatus, reqwest::Error> {
        self.client
            .get("https://api.twitter.com/1.1/application/rate_limit_status.json")
            .header(
                reqwest::header::AUTHORIZATION,
                format!("Bearer {}", self.bearer_token),
            )
            .query(&[("resources", "friends,followers,application,users")])
            .send()
            .await?
            .json::<RateLimitStatus>()
            .await
    }

    pub async fn friends_ids(&self, screen_name: &str, cursor: i64) -> Result<Ids, reqwest::Error> {
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
            .json::<Ids>()
            .await
    }

    pub async fn followers_ids(
        &self,
        screen_name: &str,
        cursor: i64,
    ) -> Result<Ids, reqwest::Error> {
        self.client
            .get("https://api.twitter.com/1.1/followers/ids.json")
            .header(
                reqwest::header::AUTHORIZATION,
                format!("Bearer {}", self.bearer_token),
            )
            .query(&[("screen_name", screen_name)])
            .query(&[("cursor", cursor)])
            .query(&[("count", 5000)])
            .send()
            .await?
            .json::<Ids>()
            .await
    }

    pub async fn users_lookup(&self, ids: &[i64]) -> Result<Vec<User>, reqwest::Error> {
        let params = ids
            .iter()
            .map(|id| id.to_string())
            .collect::<Vec<_>>()
            .join(",");

        self.client
            .get("https://api.twitter.com/1.1/users/lookup.json")
            .header(
                reqwest::header::AUTHORIZATION,
                format!("Bearer {}", self.bearer_token),
            )
            .query(&[("user_id", params)])
            .send()
            .await?
            .json::<Vec<User>>()
            .await
    }
}
