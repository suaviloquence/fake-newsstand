use std::sync::Arc;

use crate::oauth::OAuthCredentials;

mod sealed {
    pub trait ClientStateSealed {}
}

pub trait State: sealed::ClientStateSealed {}

macro_rules! seal {
    ($($T: ty $(,)?)*) => {
        $(
            impl sealed::ClientStateSealed for $T {}
			impl State for $T {}
		)*
	};
}

#[derive(Debug, Clone)]
pub struct Unauthenticated;

#[derive(Debug, Clone)]
pub struct Authenticated(pub(crate) OAuthCredentials);

#[derive(Debug, Clone)]
pub struct Temporary(pub(crate) OAuthCredentials);

seal!(Unauthenticated, Authenticated, Temporary);

#[derive(Debug)]
pub struct ClientInner<S: State> {
    client: reqwest::Client,
    oauth_consumer_key: String,
    oauth_client_secret: String,
    state: S,
}

#[derive(Debug, Clone)]
pub struct Client<S: State> {
    pub(crate) inner: Arc<ClientInner<S>>,
}

static USER_AGENT: &str = concat!(env!("CARGO_PKG_NAME"), " v", env!("CARGO_PKG_VERSION"));

impl<S: State> Client<S> {
    #[inline]
    pub(crate) fn state(&self) -> &S {
        &self.inner.state
    }

    #[inline]
    pub(crate) fn state_mut(&mut self) -> &mut S {
        &mut self.inner.state
    }

    #[inline]
    pub(crate) fn client(&self) -> &reqwest::Client {
        &self.inner.client
    }

    /// Attempts to wrap the client with the given state
    /// Returns an `Err` containing the original client and provided state if it is referenced somewhere else (i.e., [`Arc::try_unwrap`] returns `Err`)
    pub(crate) fn try_into_other_state<U: State>(self, state: U) -> Result<Client<U>, (Self, U)> {
        match Arc::try_unwrap(self.inner) {
            Ok(ClientInner {
                client,
                oauth_consumer_key,
                oauth_client_secret,
                ..
            }) => Ok(Client {
                inner: Arc::new(ClientInner {
                    client,
                    oauth_consumer_key,
                    oauth_client_secret,
                    state,
                }),
            }),
            Err(inner) => Err((Self { inner }, state)),
        }
    }
}

impl Client<Unauthenticated> {
    /// Creates a new unauthenticated `Client` with the given OAuth application keys
    pub fn new(oauth_consumer_key: String, oauth_client_secret: String) -> Self {
        Self {
            inner: Arc::new(ClientInner {
                client: reqwest::Client::builder()
                    .user_agent(USER_AGENT)
                    .https_only(true)
                    .build()
                    .expect("tumblr-api::Client::new"),
                oauth_consumer_key,
                oauth_client_secret,
                state: Unauthenticated,
            }),
        }
    }

    /// Attempts to wrap the provided credentials into an authenticated `Client`
    /// Returns an `Err` containing the `Client<Unauthenticated>` and the provided `OAuthCredentials` if the `Client` is has more than one reference to it (i.e., [`Arc::try_unwrap`] fails)
    #[inline]
    pub fn with_credentials(
        self,
        credentials: OAuthCredentials,
    ) -> Result<Client<Authenticated>, (Self, OAuthCredentials)> {
        self.try_into_other_state(Authenticated(credentials))
            .map_err(|(client, state)| (client, state.0))
    }
}
