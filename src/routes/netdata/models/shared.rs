use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct NetdataNode {
    /// Example: "la1.node.garland.slvn.co"
    pub hostname: String,
}
