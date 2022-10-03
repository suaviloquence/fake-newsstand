use serde::Deserialize;

use crate::{post::Post, AuthenticatedTumblrClient};

#[derive(Deserialize, Debug)]
pub struct BlogInfo {
	pub title: String,
	pub posts: u64,
	pub name: String,
	/// seconds from epoch
	pub updated: u64,
	pub description: String,
	/// does blog allow asks
	pub ask: bool,
	/// does blog allow anonymous asks (none if [ask][#ask] is false)
	pub ask_anon: Option<bool>,
	/// only if blog is primary blog and sharing likes is enabled
	pub likes: Option<u8>,
}

#[derive(Deserialize, Debug)]
pub struct AuthedBlogInfo {
	#[serde(flatten)]
	pub info: BlogInfo,
	pub is_blocked_from_primary: Option<bool>,
	// TODO avatar
	// TODO theme
	/// timezone as location string available only if user is a member of this blog
	pub timezone: Option<String>,
	/// timezone as offset from UTC, see above
	pub timezpme_offset: Option<String>,
}

#[derive(Debug)]
pub struct BlogClient<'a> {
	client: &'a AuthenticatedTumblrClient,
	blog_identifier: &'a str,
}

impl<'a> BlogClient<'a> {
	pub(crate) fn new(client: &'a AuthenticatedTumblrClient, blog_identifier: &'a str) -> Self {
		Self {
			client,
			blog_identifier,
		}
	}

	pub async fn info(&self) -> crate::Result<AuthedBlogInfo> {
		self.client
			.get(&format!("blog/{}/info", self.blog_identifier))
			.await
	}

	pub async fn get_posts<const limit: usize>(
		&self,
		tags: Option<Vec<String>>,
		offset: Option<u64>,
		before: Option<u64>,
	) -> crate::Result<[Post; limit]> {
		todo!()
	}
}
