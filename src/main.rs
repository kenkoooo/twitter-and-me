use twitter_and_me::client::TwitterClient;
use twitter_and_me::config::{read_json_file, write_json_file, Config};
use twitter_and_me::Result;

use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

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
        log::info!("friends={}", friends.ids.len());
        if friends.cursor != 0 {
            let mut client: Option<&TwitterClient> = None;
            for (i, c) in clients.iter().enumerate() {
                let status = c.rate_limit_status().await?;
                let remaining = status.friends_ids().expect("Failed to get /friends/ids");
                if remaining > 0 {
                    client = Some(c);
                    break;
                } else {
                    log::warn!("client {}: {:?}", i, status);
                }
            }

            if let Some(client) = client {
                let ids = client.friends_ids("kenkoooo", friends.cursor).await?;
                friends.ids.extend(ids.ids);
                friends.cursor = ids.next_cursor;
                write_json_file(friends, FRIENDS_IDS)?;
            } else {
                log::warn!("No client is available for /friends/ids");
            }
        }

        let mut followers = read_json_file::<Ids>(FOLLOWERS_IDS).unwrap_or(Ids {
            ids: vec![],
            cursor: -1,
        });
        log::info!("followers={}", followers.ids.len());
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
                    log::warn!("client {}: {:?}", i, status);
                }
            }

            if let Some(client) = client {
                let ids = client.followers_ids("kenkoooo", followers.cursor).await?;
                followers.ids.extend(ids.ids);
                followers.cursor = ids.next_cursor;
                write_json_file(followers, FOLLOWERS_IDS)?;
            } else {
                log::warn!("No client is available for /followers/ids");
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
            Err(e) => log::warn!("{:?} {:?}", config, e),
        }
    }
    Ok(clients)
}

#[tokio::main]
async fn main() -> Result<()> {
    simple_logger::init().unwrap();

    let friends = read_json_file::<Ids>(FRIENDS_IDS)?.ids;
    let followers = read_json_file::<Ids>(FOLLOWERS_IDS)?
        .ids
        .into_iter()
        .collect::<BTreeSet<_>>();
    let to_remove: Vec<_> = friends
        .into_iter()
        .rev()
        .filter(|id| !followers.contains(id))
        .collect();
    log::info!("removing: {}", to_remove.len());

    let clients = initialize_clients().await?;
    for client in clients.iter() {
        let status = client.rate_limit_status().await?;
        let remaining = status.users_lookup().expect("/users/lookup");
        if remaining == 0 {
            log::warn!("{:?}", status);
            continue;
        }
        let users = client.users_lookup(&to_remove[..100]).await?;
        for user in users.iter() {
            println!(
                "{} {}",
                user.name,
                if user.protected { "(protected)" } else { "" }
            );
            println!(
                "Friends: {}, Followers: {}",
                user.friends_count, user.followers_count
            );
            println!("{}", user.description);
            println!("https://twitter.com/{}", user.screen_name);
            println!();
        }
        break;
    }
    Ok(())
}
