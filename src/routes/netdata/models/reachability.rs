use crate::routes::netdata::NetdataNode;
use serde::{Deserialize, Serialize};
use serde_with::DeserializeFromStr;
use strum::{Display, EnumString};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct NetdataReachabilityPayload {
    /// Example: "Slvn.co/sa1.node.garland.slvn.co is unreachable"
    pub message: String,
    /// Example: "https://app.netdata.cloud/..."
    pub url: String,
    /// Example: "critical"
    pub severity: NetdataReachabilitySeverity,
    /// Example: "unreachable"
    pub status: NetdataReachabilityStatus,
    pub nodes: Vec<NetdataNode>,
}

#[derive(Serialize, DeserializeFromStr, EnumString, Display, Debug, Clone, Copy, PartialEq)]
#[strum(ascii_case_insensitive, serialize_all = "lowercase")]
pub enum NetdataReachabilityStatus {
    Reachable,
    Unreachable,
}

#[derive(Serialize, DeserializeFromStr, EnumString, Display, Debug, Clone, Copy, PartialEq)]
#[strum(ascii_case_insensitive, serialize_all = "lowercase")]
pub enum NetdataReachabilitySeverity {
    Info,
    Critical,
}
