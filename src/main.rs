use std::env;

use anyhow::Context;
use tokio::io::{self, AsyncBufReadExt, BufReader};
use tumblr_api::{oauth::OAuthCredentials, Authenticated, Client};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    if let Err(e) = dotenv::dotenv() {
        eprintln!(
            "Error loading environment variables from .env file: {:?}",
            e
        );
    }

    let oauth_consumer_key = env::var("TUMBLR_OAUTH_CONSUMER_KEY")
        .context("Missing environment variable TUMBLR_OAUTH_CONSUMER_KEY")?;
    let oauth_client_secret = env::var("TUMBLR_OAUTH_CLIENT_SECRET")
        .context("Missing environment variable TUMBLR_OAUTH_CLIENT_SECRET")?;

    let oauth_token = env::var("TUMBLR_OAUTH_TOKEN").ok();
    let oauth_token_secret = env::var("TUMBLR_OAUTH_TOKEN_SECRET").ok();

    let client = Client::new(oauth_consumer_key, oauth_client_secret);

    let client = {
        if let (Some(oauth_token), Some(oauth_token_secret)) = (oauth_token, oauth_token_secret) {
            client
                .with_credentials(OAuthCredentials {
                    oauth_token,
                    oauth_token_secret,
                })
                .unwrap()
        } else {
            let client = TumblrClient::create(oauth_consumer_key, oauth_client_secret)?;

            let credentials = client.create_temporary_credentials().await?;

            println!("{:?}", credentials);

            println!("{}", client.generate_callback_url(&credentials));

            let mut url = String::new();

            let mut reader = BufReader::new(io::stdin());

            reader.read_line(&mut url).await?;

            let oauth_verifier = client
                .parse_redirect_url(&url[..url.find("\n").unwrap()])
                .unwrap();
            client.verify_token(credentials, oauth_verifier).await?
        }
    };

    let blog = client.blog("adawed");

    println!("{:?}");

    println!("\n\n\n\n\n");

    Ok(())
}
