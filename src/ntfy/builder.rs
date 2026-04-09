use crate::config::NtfyCredentials;
use crate::ntfy::client::NtfyClientShared;
use crate::ntfy::http_client::NtfyHttpClient;
use ntfy::auth::Auth;
use ntfy::dispatcher;
use std::sync::Arc;

pub struct NtfyClientBuilder {
    url: String,
    credentials: NtfyCredentials,
}

impl NtfyClientBuilder {
    pub fn new(url: impl AsRef<str>, credentials: NtfyCredentials) -> Self {
        Self {
            url: url.as_ref().to_owned(),
            credentials,
        }
    }

    pub fn build(self) -> anyhow::Result<NtfyClientShared> {
        let mut builder = dispatcher::builder(self.url);

        match self.credentials {
            NtfyCredentials::AuthToken(token) => {
                builder = builder.credentials(Auth::token(token));
            }
            NtfyCredentials::UsernamePassword(username, password) => {
                builder = builder.credentials(Auth::credentials(username, password));
            }
            NtfyCredentials::None => {}
        }

        let dispatcher = builder.build_async()?;
        Ok(Arc::new(NtfyHttpClient { dispatcher }))
    }
}
