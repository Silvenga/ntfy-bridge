use super::{NetdataAlertPayload, NetdataReachabilityPayload, NetdataTokenPayload};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(untagged)]
pub enum NetdataPayload {
    Token(NetdataTokenPayload),
    Alert(NetdataAlertPayload),
    Reachability(NetdataReachabilityPayload),
}

#[cfg(test)]
mod tests {
    use super::super::{
        NetdataAlertStatus, NetdataReachabilitySeverity, NetdataReachabilityStatus,
    };
    use super::*;
    use serde_json::json;

    #[test]
    fn when_parsing_alert_payload_then_should_succeed() {
        let data = json!({
            "message": "Raised to Critical, System swap memory utilization = 100%, on la1.node.garland.slvn.co",
            "space": {
                "name": "Slvn.co"
            },
            "node": {
                "hostname": "la1.node.garland.slvn.co"
            },
            "alert": {
                "name": "used_swap",
                "state": {
                    "status": "Critical"
                },
                "rendered": {
                    "info": "Swap memory utilization"
                },
                "url": "https://app.netdata.cloud/spaces/slvn-co/rooms/all-nodes/alerts/mem.swap.used_swap:::e84d4453-cb1f-48b5-abfd-0a82629712bd?chart=mem.swap&alarm=used_swap&transition=ee19d16d-e3d5-43f3-b313-c9f7ee6fc7f8",
                "config": {
                    "classification": "Utilization"
                }
            }
        });

        let payload: NetdataPayload =
            serde_json::from_value(data).expect("should have parsed alert payload");

        match payload {
            NetdataPayload::Alert(alert) => {
                assert_eq!(
                    alert.message,
                    "Raised to Critical, System swap memory utilization = 100%, on la1.node.garland.slvn.co"
                );
                assert_eq!(alert.alert.name, "used_swap");
                assert_eq!(alert.alert.state.status, NetdataAlertStatus::Critical);
                assert_eq!(alert.node.hostname, "la1.node.garland.slvn.co");
            }
            _ => panic!("should have been an alert payload"),
        }
    }

    #[test]
    fn when_parsing_alert_status_case_insensitive_then_should_succeed() {
        let statuses = vec![
            "critical", "CRITICAL", "Critical", "warning", "WARNING", "Warning", "clear", "CLEAR",
            "Clear",
        ];
        for status_raw in statuses {
            let data = json!(status_raw);
            let status: NetdataAlertStatus =
                serde_json::from_value(data).expect("should have parsed status");
            match status_raw.to_lowercase().as_str() {
                "critical" => assert_eq!(status, NetdataAlertStatus::Critical),
                "warning" => assert_eq!(status, NetdataAlertStatus::Warning),
                "clear" => assert_eq!(status, NetdataAlertStatus::Clear),
                _ => unreachable!(),
            }
        }
    }

    #[test]
    fn when_parsing_reachability_status_case_insensitive_then_should_succeed() {
        let statuses = vec![
            "reachable",
            "REACHABLE",
            "Reachable",
            "unreachable",
            "UNREACHABLE",
            "Unreachable",
        ];
        for status_raw in statuses {
            let data = json!(status_raw);
            let status: NetdataReachabilityStatus =
                serde_json::from_value(data).expect("should have parsed reachability status");
            match status_raw.to_lowercase().as_str() {
                "reachable" => assert_eq!(status, NetdataReachabilityStatus::Reachable),
                "unreachable" => assert_eq!(status, NetdataReachabilityStatus::Unreachable),
                _ => unreachable!(),
            }
        }
    }

    #[test]
    fn when_parsing_reachability_severity_case_insensitive_then_should_succeed() {
        let severities = vec!["info", "INFO", "Info", "critical", "CRITICAL", "Critical"];
        for severity_raw in severities {
            let data = json!(severity_raw);
            let severity: NetdataReachabilitySeverity =
                serde_json::from_value(data).expect("should have parsed reachability severity");
            match severity_raw.to_lowercase().as_str() {
                "info" => assert_eq!(severity, NetdataReachabilitySeverity::Info),
                "critical" => assert_eq!(severity, NetdataReachabilitySeverity::Critical),
                _ => unreachable!(),
            }
        }
    }

    #[test]
    fn when_parsing_reachability_payload_then_should_succeed() {
        let data = json!({
            "message": "Slvn.co/sa1.node.garland.slvn.co is unreachable",
            "url": "https://app.netdata.cloud/spaces/slvn-co/rooms/all-nodes/nodes/6c7cc3a5-33d8-4b02-b882-ebd04e8c6009",
            "severity": "critical",
            "status": "unreachable",
            "nodes": [
                {
                    "hostname": "sa1.node.garland.slvn.co"
                }
            ]
        });

        let payload: NetdataPayload =
            serde_json::from_value(data).expect("should have parsed reachability payload");

        match payload {
            NetdataPayload::Reachability(reach) => {
                assert_eq!(
                    reach.message,
                    "Slvn.co/sa1.node.garland.slvn.co is unreachable"
                );
                assert_eq!(reach.status, NetdataReachabilityStatus::Unreachable);
                assert_eq!(reach.severity, NetdataReachabilitySeverity::Critical);
                assert_eq!(reach.nodes[0].hostname, "sa1.node.garland.slvn.co");
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
            NetdataPayload::Token(token) => {
                assert_eq!(
                    token.message,
                    "This is a Test Notification. If you're receiving it, your Webhook integration is configured properly!"
                );
                assert_eq!(token.title, "Test Notification");
                assert_eq!(token.token, "2b633082-34ec-4ec3-946c-5e81076c39af");
            }
            _ => panic!("should have been a token payload"),
        }
    }
}
