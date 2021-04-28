//! Module that contains the json types binding used to communicate with a cosmos based blockchain node.
use serde::{Deserialize, Serialize};

/// NodeInfoResponse contains the response of the LCD request `/node_info`.
#[derive(Clone, Serialize, Deserialize)]
pub struct NodeInfoResponse {
    pub node_info: NodeInfo,
}

/// NodeInfo contains the information of a cosmos based blockchain node.
#[derive(Clone, Serialize, Deserialize)]
pub struct NodeInfo {
    pub id: String,
    pub listen_addr: String,
    pub network: String,
    pub version: String,
    pub moniker: String,
}
