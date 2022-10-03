pub use client::{Authenticated, Client, Temporary, Unauthenticated};
pub use error::Error;
pub use oauth::OAuthCredentials;

use reqwest::{Method, Url};
use serde::{de::DeserializeOwned, Deserialize};

use self::post::Post;

pub mod blog;
mod client;
mod error;
pub mod oauth;
pub mod post;

pub use error::Result;

#[derive(Debug, Deserialize)]
pub struct ResponseMeta {
    pub status: u16,
    pub msg: String,
}

#[derive(Deserialize, Debug)]
struct Response<T> {
    meta: ResponseMeta,
    response: Option<T>,
}

impl Client<Authenticated> {
    pub const API_BASE: &'static str = "https://api.tumblr.com/v2";

    pub(crate) async fn request<T: DeserializeOwned>(
        &self,
        method: Method,
        path: &str,
        data: Option<()>,
    ) -> Result<T> {
        let url: Url = format!("{}/{}", Self::API_BASE, path).parse()?;

        let mut req = self.client().request(method, url);

        if let Some(data) = data {
            todo!()
        }

        let req = req.build()?.sign(
            &self.oauth_consumer_key,
            &self.oauth_client_secret,
            Some(&self.credentials.oauth_token),
            Some(&self.credentials.oauth_token_secret),
            vec![],
        );

        let res = self.client.execute(req).await?;
        let text = res.text().await?;

        let res: Response<T> = serde_json::from_str(&text)?;

        if res.meta.status == 200 {
            Ok(res.response.unwrap())
        } else {
            Err(Error::Tumblr(res.meta))
        }
    }

    pub(crate) async fn get<T: DeserializeOwned>(&self, path: &str) -> Result<T> {
        self.request(Method::GET, path, None).await
    }

    pub(crate) async fn post<T: DeserializeOwned>(
        &self,
        path: &str,
        data: Option<()>,
    ) -> Result<T> {
        self.request(Method::POST, path, data).await
    }

    pub async fn blog_info(&self, blog_identifier: &str) -> Result<serde_json::Value> {
        self.request(Method::GET, &format!("blog/{blog_identifier}/info"), None)
            .await
    }

    pub async fn get_post(&self, post_id: u64) -> Result<serde_json::Value> {
        self.request(Method::GET, &format!("posts/{post_id}"), None)
            .await
    }

    pub async fn create_post(&self, blog_name: &str, post: Post) -> Result<bool> {
        todo!()
    }
}
