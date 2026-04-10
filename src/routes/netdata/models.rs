use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(untagged)]
pub enum NetdataPayload {
    Token(NetdataTokenPayload),
    Alert(Box<NetdataAlertPayload>),
    Reachability(NetdataReachabilityPayload),
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct NetdataTokenPayload {
    pub message: String,
    pub title: String,
    pub token: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct NetdataAlertPayload {
    pub message: String,
    pub alert: String,
    pub info: String,
    pub chart: String,
    pub context: String,
    pub space: String,
    pub rooms: HashMap<String, String>,
    pub family: String,
    pub class: String,
    pub severity: String,
    pub date: String,
    pub duration: String,
    pub additional_active_critical_alerts: u32,
    pub additional_active_warning_alerts: u32,
    pub alert_url: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct NetdataReachabilityPayload {
    pub message: String,
    pub url: String,
    pub host: String,
    pub severity: String,
    pub status: NetdataReachabilityStatus,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct NetdataReachabilityStatus {
    pub reachable: bool,
    pub text: String,
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn when_parsing_alert_payload_then_should_succeed() {
        let data = json!({
            "message": "test message",
            "alert": "test alert",
            "info": "test info",
            "chart": "test chart",
            "context": "test context",
            "space": "test space",
            "rooms": {
                "room1": "url1"
            },
            "family": "test family",
            "class": "Error",
            "severity": "critical",
            "date": "2024-01-01T00:00:00Z",
            "duration": "1m",
            "additional_active_critical_alerts": 1,
            "additional_active_warning_alerts": 2,
            "alert_url": "http://example.com"
        });

        let payload: NetdataPayload =
            serde_json::from_value(data).expect("should have parsed alert payload");

        match payload {
            NetdataPayload::Alert(alert) => {
                assert_eq!(alert.message, "test message");
                assert_eq!(alert.alert, "test alert");
                assert_eq!(alert.severity, "critical");
                assert_eq!(
                    alert.rooms.get("room1").expect("room1 should exist"),
                    "url1"
                );
            }
            _ => panic!("should have been an alert payload"),
        }
    }

    #[test]
    fn when_parsing_reachability_payload_then_should_succeed() {
        let data = json!({
            "message": "host is unreachable",
            "url": "http://example.com",
            "host": "test-host",
            "severity": "critical",
            "status": {
                "reachable": false,
                "text": "unreachable"
            }
        });

        let payload: NetdataPayload =
            serde_json::from_value(data).expect("should have parsed reachability payload");

        match payload {
            NetdataPayload::Reachability(reach) => {
                assert_eq!(reach.message, "host is unreachable");
                assert_eq!(reach.host, "test-host");
                assert!(!reach.status.reachable);
            }
            _ => panic!("should have been a reachability payload"),
        }
    }

    #[test]
    fn when_parsing_token_payload_then_should_succeed() {
        let data = json!({
            "message": "This is a Test Notification. If you're receiving it, your Webhook integration is configured properly!",
            "title": "Test Notification",
            "token": "2b633082-34ec-4ec3-946c-5e81076c39af"
        });

        let payload: NetdataPayload =
            serde_json::from_value(data).expect("should have parsed reachability payload");

        match payload {
            NetdataPayload::Token(reach) => {
                assert_eq!(
                    reach.message,
                    "This is a Test Notification. If you're receiving it, your Webhook integration is configured properly!"
                );
                assert_eq!(reach.title, "Test Notification");
                assert_eq!(reach.token, "2b633082-34ec-4ec3-946c-5e81076c39af");
            }
            _ => panic!("should have been a token payload"),
        }
    }
}
