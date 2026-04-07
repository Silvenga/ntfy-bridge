use crate::ntfy::http_client::NtfyHttpClient;
use ntfy::auth::Auth;
use ntfy::dispatcher;

pub struct NtfyHttpClientBuilder {
    url: String,
    credentials: Option<Auth>,
}

impl NtfyHttpClientBuilder {
    pub fn new(url: impl AsRef<str>) -> Self {
        Self {
            url: url.as_ref().to_owned(),
            credentials: None,
        }
    }

    pub fn with_credentials(
        mut self,
        username: impl AsRef<str>,
        password: impl AsRef<str>,
    ) -> Self {
        self.credentials = Some(Auth::credentials(
            username.as_ref().to_owned(),
            password.as_ref().to_owned(),
        ));
        self
    }

    pub fn with_token(mut self, token: impl AsRef<str>) -> Self {
        self.credentials = Some(Auth::token(token.as_ref().to_owned()));
        self
    }

    pub fn build(self) -> Result<NtfyHttpClient, ntfy::Error> {
        let mut builder = dispatcher::builder(self.url);
        if let Some(auth) = self.credentials {
            builder = builder.credentials(auth);
        }
        let dispatcher = builder.build_async()?;
        Ok(NtfyHttpClient { dispatcher })
    }
}
