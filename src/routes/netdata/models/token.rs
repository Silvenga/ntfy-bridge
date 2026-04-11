use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct NetdataTokenPayload {
    /// Example: "This is a Test Notification. If you're receiving it, your Webhook integration is configured properly!"
    pub message: String,
    /// Example: "Test Notification"
    pub title: String,
    /// Example: "2b633082-34ec-4ec3-946c-5e81076c39af"
    pub token: String,
}
