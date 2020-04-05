use twitter_and_me::client::TwitterClient;
use twitter_and_me::io::{read_json_file, write_html_file, write_json_file, Config};
use twitter_and_me::Result;

use askama::Template;
use log::{info, warn};
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;
use twitter_and_me::model::User;

const TWITTER_CONF: &str = ".twitter.json";
const FRIENDS_IDS: &str = "friends_ids.json";
const FOLLOWERS_IDS: &str = "followers_ids.json";

#[derive(Deserialize, Serialize)]
struct Ids {
    ids: Vec<i64>,
    cursor: i64,
}

async fn load_ff(clients: &Vec<TwitterClient>) -> Result<()> {
    loop {
        let mut friends = read_json_file::<Ids>(FRIENDS_IDS).unwrap_or(Ids {
            ids: vec![],
            cursor: -1,
        });
        info!("friends={}", friends.ids.len());
        if friends.cursor != 0 {
            let mut client: Option<&TwitterClient> = None;
            for (i, c) in clients.iter().enumerate() {
                let status = c.rate_limit_status().await?;
                let remaining = status.friends_ids().expect("Failed to get /friends/ids");
                if remaining > 0 {
                    client = Some(c);
                    break;
                } else {
                    warn!("client {}: {:?}", i, status);
                }
            }

            if let Some(client) = client {
                let ids = client.friends_ids("kenkoooo", friends.cursor).await?;
                friends.ids.extend(ids.ids);
                friends.cursor = ids.next_cursor;
                write_json_file(friends, FRIENDS_IDS)?;
            } else {
                warn!("No client is available for /friends/ids");
            }
        }

        let mut followers = read_json_file::<Ids>(FOLLOWERS_IDS).unwrap_or(Ids {
            ids: vec![],
            cursor: -1,
        });
        info!("followers={}", followers.ids.len());
        if followers.cursor != 0 {
            let mut client: Option<&TwitterClient> = None;
            for (i, c) in clients.iter().enumerate() {
                let status = c.rate_limit_status().await?;
                let remaining = status
                    .followers_ids()
                    .expect("Failed to get /followers/ids");
                if remaining > 0 {
                    client = Some(c);
                    break;
                } else {
                    warn!("client {}: {:?}", i, status);
                }
            }

            if let Some(client) = client {
                let ids = client.followers_ids("kenkoooo", followers.cursor).await?;
                followers.ids.extend(ids.ids);
                followers.cursor = ids.next_cursor;
                write_json_file(followers, FOLLOWERS_IDS)?;
            } else {
                warn!("No client is available for /followers/ids");
            }
        }
    }
}

async fn initialize_clients() -> Result<Vec<TwitterClient>> {
    let configs = read_json_file::<Vec<Config>>(TWITTER_CONF)?;
    let mut clients = vec![];
    for config in configs.into_iter() {
        match TwitterClient::new(&config.key, &config.secret).await {
            Ok(client) => clients.push(client),
            Err(e) => warn!("{:?} {:?}", config, e),
        }
    }
    Ok(clients)
}

#[derive(Template)]
#[template(path = "users.html")]
struct UsersTemplate {
    users: Vec<User>,
}

async fn lookup_users(clients: &Vec<TwitterClient>, mut ids: Vec<i64>) -> Result<Vec<User>> {
    let mut users: Vec<User> = vec![];
    let mut fetch_count = 0;
    while !ids.is_empty() && fetch_count < 10 {
        let ids = ids.pop_n(100);

        for client in clients.iter() {
            let status = client.rate_limit_status().await?;
            let remaining = status.users_lookup().expect("/users/lookup");
            if remaining == 0 {
                warn!("{:?}", status);
                continue;
            }

            info!("Fetching {} users ...", ids.len());
            let u = client.users_lookup(&ids).await?;
            users.extend(u);
            fetch_count += 1;
            break;
        }
    }
    Ok(users)
}

fn sub_set(id_file: &str, already_file: &str) -> Result<Vec<i64>> {
    let friends = read_json_file::<Ids>(already_file)?
        .ids
        .into_iter()
        .collect::<BTreeSet<_>>();
    let to_follow = read_json_file::<Ids>(id_file)?
        .ids
        .into_iter()
        .rev()
        .filter(|id| !friends.contains(&id))
        .collect::<Vec<_>>();
    Ok(to_follow)
}

async fn generate_follow_list(clients: &Vec<TwitterClient>) -> Result<()> {
    let to_follow = sub_set(FOLLOWERS_IDS, FRIENDS_IDS)?;
    info!("following: {}", to_follow.len());
    let users = lookup_users(clients, to_follow).await?;
    let users = users
        .into_iter()
        .filter(|user| !user.protected && user.statuses_count > 5)
        .collect();
    let html = UsersTemplate { users };
    write_html_file(html, "follow_list.html")
}
async fn generate_remove_list(clients: &Vec<TwitterClient>) -> Result<()> {
    let to_remove = sub_set(FRIENDS_IDS, FOLLOWERS_IDS)?;
    info!("removing: {}", to_remove.len());
    let users = lookup_users(clients, to_remove).await?;
    let users = users.into_iter().collect();
    let html = UsersTemplate { users };
    write_html_file(html, "remove_list.html")
}

#[tokio::main]
async fn main() -> Result<()> {
    simple_logger::init_with_level(log::Level::Info).unwrap();
    let clients = initialize_clients().await?;

    // generate_remove_list(&clients).await?;
    generate_follow_list(&clients).await?;

    Ok(())
}

trait VecExt<T> {
    fn pop_n(&mut self, n: usize) -> Vec<T>;
}

impl<T> VecExt<T> for Vec<T> {
    fn pop_n(&mut self, n: usize) -> Vec<T> {
        let mut result = vec![];
        while let Some(v) = self.pop() {
            result.push(v);
            if result.len() == n {
                break;
            }
        }
        result
    }
}
