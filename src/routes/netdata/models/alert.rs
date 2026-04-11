use crate::routes::netdata::models::shared::NetdataNode;
use serde::{Deserialize, Serialize};
use serde_with::DeserializeFromStr;
use strum::EnumString;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct NetdataAlertPayload {
    /// Example: "Raised to Critical, System swap memory utilization = 100%, on la1.node.garland.slvn.co"
    pub message: String,
    pub space: NetdataSpace,
    pub node: NetdataNode,
    pub alert: NetdataAlert,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct NetdataSpace {
    /// Example: "Slvn.co"
    pub name: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct NetdataAlert {
    /// Example: "used_swap"
    pub name: String,
    pub state: NetdataAlertState,
    pub rendered: NetdataAlertRendered,
    /// Example: "https://app.netdata.cloud/..."
    pub url: String,
    pub config: NetdataAlertConfig,
}

#[derive(Serialize, DeserializeFromStr, EnumString, Debug, Clone, Copy, PartialEq)]
#[strum(ascii_case_insensitive)]
pub enum NetdataAlertStatus {
    Critical,
    Warning,
    Clear,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct NetdataAlertState {
    /// Example: "Critical"
    pub status: NetdataAlertStatus,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct NetdataAlertRendered {
    /// Example: "Swap memory utilization"
    pub info: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct NetdataAlertConfig {
    /// Example: "Utilization"
    pub classification: String,
}
