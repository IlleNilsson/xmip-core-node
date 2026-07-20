#![forbid(unsafe_code)]

use std::collections::BTreeSet;
use xmip_core::NodeId;

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum NodeRole {
    Operational,
    Monitoring,
    Executing,
    Development,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Node {
    pub node_id: NodeId,
    pub name: String,
    pub roles: BTreeSet<NodeRole>,
    pub capabilities: BTreeSet<String>,
    pub trusted: bool,
}

impl Node {
    pub fn supports(&self, capability: &str) -> bool {
        self.capabilities.contains(capability)
    }
}
