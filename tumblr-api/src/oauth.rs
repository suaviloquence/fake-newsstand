use std::{cmp::Ordering, time::SystemTime};

use hmac::{Hmac, Mac};
use rand::Rng;
use reqwest::Url;
use serde::Deserialize;
use sha1::Sha1;

use crate::{client::State, Authenticated, Client, Temporary, Unauthenticated};

#[derive(Deserialize, Debug)]
pub struct OAuthCredentials {
	pub oauth_token: String,
	pub oauth_token_secret: String,
}

fn oauth_encode(str: &str) -> String {
	let mut out = String::new();

	for ch in str.chars() {
		match ch {
			'a'..='z' | 'A'..='Z' | '0'..='9' | '-' | '.' | '_' | '~' => out.push(ch),
			other => {
				let mut buf = [0; 4];
				let len = other.encode_utf8(&mut buf).len();

				for i in 0..len {
					out.push_str(&format!("%{:X}", buf[i]));
				}
			}
		}
	}

	out
}

fn generate_nonce() -> String {
	let mut rng = rand::thread_rng();

	let mut buf = [0u8; 8];

	rng.fill(&mut buf);

	// TODO check if nonce has already been used

	base64::encode(buf)
}

pub(crate) struct Request<S: State> {
	inner: reqwest::Request,
	client: Client<S>,
}

impl<S: State> AsRef<reqwest::Request> for Request<S> {
	#[inline]
	fn as_ref(&self) -> &reqwest::Request {
		&self.inner
	}
}

impl<S: State> AsMut<reqwest::Request> for Request<S> {
	#[inline]
	fn as_mut(&mut self) -> &mut reqwest::Request {
		&mut self.inner
	}
}

impl<S: State> Request<S> {
	pub(crate) fn sign(
		self,
		oauth_token: Option<&str>,
		oauth_token_secret: Option<&str>,
		other_params: Option<Vec<(&str, &str)>>,
	) -> crate::Result<Self> {
		let method = self.method().to_string().to_uppercase();

		let mut url = self.url().clone();
		let mut params: Vec<_> = url
			.query_pairs()
			.map(|(k, v)| (oauth_encode(&k), oauth_encode(&v)))
			.collect();
		url.set_query(None);
		let base_uri = url.as_str().to_lowercase();

		let timestamp = SystemTime::now()
			.duration_since(SystemTime::UNIX_EPOCH)
			.expect("it is before 1/1/1970")
			.as_secs()
			.to_string();

		let nonce = generate_nonce();

		let oauth_params: Vec<_> = vec![
			(
				"oauth_consumer_key",
				self.client.oauth_consumer_key.as_str(),
			),
			("oauth_token", oauth_token.unwrap_or("")),
			("oauth_signature_method", "HMAC-SHA1"),
			("oauth_timestamp", &timestamp),
			("oauth_nonce", &nonce),
			("oauth_version", "1.0"),
		]
		.into_iter()
		.chain(other_params.unwrap_or_default())
		.map(|(k, v)| (oauth_encode(k), oauth_encode(v)))
		.collect();

		let mut authorization: Vec<_> = oauth_params
			.iter()
			.map(|(k, v)| format!(r#"{k}="{v}""#))
			.collect();

		params.extend(
			oauth_params
				.into_iter()
				.map(|(k, v)| (k.to_owned(), v.to_owned())),
		);

		params.sort_by(|(a, b), (c, d)| match a.cmp(c) {
			Ordering::Equal => b.cmp(d),
			other => other,
		});

		let params = params
			.into_iter()
			.map(|(k, v)| format!("{k}={v}"))
			.collect::<Vec<_>>()
			.join("&");

		let base_string = format!(
			"{}&{}&{}",
			oauth_encode(&method),
			oauth_encode(&base_uri),
			oauth_encode(&params),
		);

		let secret = format!(
			"{}&{}",
			oauth_encode(&self.client.oauth_client_secret),
			oauth_encode(oauth_token_secret.unwrap_or(""))
		);

		let mut hmac =
			HmacSha1::new_from_slice(secret.as_bytes()).expect("Invalid length of secret key");

		hmac.update(base_string.as_bytes());

		let oauth_signature = oauth_encode(&base64::encode(hmac.finalize().into_bytes()));

		authorization.push(format!(r#"oauth_signature="{oauth_signature}""#));

		self.headers_mut().append(
			reqwest::header::AUTHORIZATION,
			format!("OAuth {}", authorization.join(","))
				.parse()
				.expect("Invalid header value"),
		);

		Ok(self)
	}

	pub(crate) async fn sign_and_send(
		self,
		oauth_token: Option<&str>,
		oauth_token_secret: Option<&str>,
		other_params: Option<Vec<(&str, &str)>>,
	) -> crate::Result<reqwest::Response> {
		self.client
			.client
			.execute(
				self.sign(oauth_token, oauth_token_secret, other_params)?
					.inner,
			)
			.await
			.map_err(|err| crate::Error::Http(err))
	}
}

pub(crate) trait SignRequest {
	fn sign(
		self,
		oauth_consumer_key: &str,
		oauth_client_secret: &str,
		oauth_token: Option<&str>,
		oauth_token_secret: Option<&str>,
		other_params: Vec<(&str, &str)>,
	) -> Self;
}

type HmacSha1 = Hmac<Sha1>;

impl SignRequest for Request {
	fn sign(
		mut self,
		oauth_consumer_key: &str,
		oauth_client_secret: &str,
		oauth_token: Option<&str>,
		oauth_token_secret: Option<&str>,
		other_params: Vec<(&str, &str)>,
	) -> Self {
		self
	}
}

impl Client<Unauthenticated> {
	async fn create_temporary_credentials(self) -> crate::Result<OAuthCredentials> {
		let res = self
			.client
			.execute(
				self.client
					.post("https://www.tumblr.com/oauth/request_token")
					.build()?
					.sign(
						&self.oauth_consumer_key,
						&self.oauth_client_secret,
						None,
						None,
						// vec![("oauth_callback", "oob")],
						vec![],
					),
			)
			.await?;

		let text = res.text().await?;

		serde_urlencoded::from_str(&text).map_err(crate::Error::DeserializeForm)
	}

	pub async fn try_into_temporary(
		self,
	) -> crate::Result<Result<Client<Temporary>, (Self, OAuthCredentials)>> {
		let credentials = self.create_temporary_credentials().await?;

		Ok(self
			.try_into_other_state(Temporary(credentials))
			.map_err(|(client, state)| (client, state.0)))
	}
}

impl Client<Temporary> {
	fn generate_callback_url(&self) -> String {
		format!(
			"https://www.tumblr.com/oauth/authorize?oauth_token={}",
			&self.state().0.oauth_token
		)
	}

	fn parse_redirect_url(url: &str) -> Option<String> {
		let parsed: Url = url.parse().ok()?;

		parsed
			.query_pairs()
			.find(|(k, _)| k == "oauth_verifier")
			.map(|(_, v)| v.into_owned())
	}

	pub async fn verify_token(
		self,
		temporary_credentials: OAuthCredentials,
		oauth_verifier: String,
	) -> crate::Result<Result<Client<Authenticated>, (Self, OAuthCredentials)>> {
		let res = self
			.client
			.execute(
				self.client
					.post("https://www.tumblr.com/oauth/access_token")
					.build()?
					.sign(
						&self.oauth_consumer_key,
						&self.oauth_client_secret,
						Some(&temporary_credentials.oauth_token),
						Some(&temporary_credentials.oauth_token_secret),
						vec![("oauth_verifier", &oauth_verifier)],
					),
			)
			.await?;

		let credentials = serde_urlencoded::from_str(&res.text().await?)?;

		Ok(self
			.try_into_other_state(Authenticated(credentials))
			.map_err(|(client, state)| (client, state.0)))
	}
}
