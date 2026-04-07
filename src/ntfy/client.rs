use async_trait::async_trait;
use ntfy::Payload;
use std::sync::Arc;

#[cfg_attr(test, mockall::automock)]
#[async_trait]
pub trait NtfyClient: Send + Sync {
    async fn send(&self, payload: &Payload) -> Result<(), ntfy::Error>;
}

pub type NtfyClientShared = Arc<dyn NtfyClient>;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ntfy::client::MockNtfyClient;
    use ntfy::Priority;

    #[tokio::test]
    async fn when_send_payload_then_should_be_received_by_client() {
        let mut mock = MockNtfyClient::new();
        let payload = Payload::new("test_topic")
            .message("test message")
            .title("test title")
            .priority(Priority::High);
        let expected_payload = payload.clone();

        mock.expect_send()
            .withf(move |p| {
                p.topic == expected_payload.topic
                    && p.message == expected_payload.message
                    && p.title == expected_payload.title
                    && p.priority == expected_payload.priority
            })
            .times(1)
            .returning(|_| Ok(()));

        mock.send(&payload).await.expect("should have sent payload");
    }
}
