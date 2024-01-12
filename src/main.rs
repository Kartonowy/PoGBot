use futures::StreamExt;
use roux::{Subreddit, Reddit};
use roux_stream::stream_comments;
use std::time::Duration;
use tokio_retry::strategy::ExponentialBackoff;
use std::env;
use dotenvy::{self, dotenv};


#[tokio::main]
async fn main() {
    let subreddit = Subreddit::new("wizardposting");
    dotenv().expect(".env file not found");
    // How often to retry when pulling the data from Reddit fails and
    // how long to wait between retries. See the docs of `tokio_retry`
    // for details.
    let client = Reddit::new("reddit bot by u/Kartonek124", &env::var("REDDIT_PUBLIC_KEY").unwrap(), &env::var("REDDIT_SECRET_KEY").unwrap())
        .username(&env::var("REDDIT_USERNAME").unwrap())
        .password(&env::var("REDDIT_PASSWORD").unwrap())
        .login()
        .await;
    let me = client.unwrap();
    let retry_strategy = ExponentialBackoff::from_millis(5).factor(100).take(3);

    let (mut stream, join_handle) = stream_comments(
        &subreddit,
        Duration::from_secs(10),
        retry_strategy,
        Some(Duration::from_secs(10)),
        );
    
    let trigger_words = ["YUGI", "MTG", "YUGIOH", "YU-GI-OH", "DUEL"];

    while let Some(comment) = stream.next().await {
        // `comment` is an `Err` if getting the latest comments
        // from Reddit failed even after retrying.
        let comment = comment.unwrap();
        for word in trigger_words {
            if comment.body.clone().unwrap().to_uppercase().contains(word) {
                let _res = me.comment("#A DUEL?! FINE. MY TURN!\n\n__I SUMMON POT OF GREED TO DRAW THREE ADDITIONAL CARDS FROM MY DECK!!__", &comment.name.clone().unwrap().replace("\"", "")).await.expect("Failed to comment.");
            }
        }
    }


    // In case there was an error sending the submissions through the
    // stream, `join_handle` will report it.
    join_handle.await.unwrap().unwrap();
}
