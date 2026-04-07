use crate::ntfy::client::NtfyClient;
use async_trait::async_trait;
use ntfy::dispatcher::Async;
use ntfy::{Dispatcher, Payload};

pub struct NtfyHttpClient {
    pub(crate) dispatcher: Dispatcher<Async>,
}

#[async_trait]
impl NtfyClient for NtfyHttpClient {
    async fn send(&self, payload: &Payload) -> Result<(), ntfy::Error> {
        self.dispatcher.send(payload).await
    }
}
