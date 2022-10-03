use thiserror::Error;

use crate::ResponseMeta;

#[derive(Debug, Error)]
pub enum Error {
	#[error("Tumblr API error: {0:?}")]
	Tumblr(ResponseMeta),
	#[error("HTTP error")]
	Http(#[from] reqwest::Error),
	#[error("Error parsing URL")]
	UrlParse(#[from] url::ParseError),
	#[error("Error deserializing response JSON")]
	DeserializeJson(#[from] serde_json::Error),
	#[error("Error deserializing response form data")]
	DeserializeForm(#[from] serde_urlencoded::de::Error),
}

pub type Result<T> = std::result::Result<T, Error>;
