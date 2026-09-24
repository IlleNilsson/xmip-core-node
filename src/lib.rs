#![forbid(unsafe_code)]

pub mod declaration;
pub mod stage;

pub use declaration::{Declaration, Declared, Purpose};
pub use stage::Stage;

use std::collections::BTreeSet;

use serde::{Deserialize, Serialize};
use xcore::NodeId;

/// What a node is for. The one declaration; the runtime's host plan carried
/// a copy of these four variants until 2026-09-14 and now uses this one
/// (ADR-0044: shared code lives where both already depend). Kebab-case is
/// the form the plan already wrote.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum NodeRole {
    Operational,
    Monitoring,
    Executing,
    Development,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct Node {
    pub node_id: NodeId,
    pub name: String,
    pub roles: BTreeSet<NodeRole>,
    pub capabilities: BTreeSet<String>,
    pub trusted: bool,
}

impl Node {
    #[must_use]
    pub fn supports(&self, capability: &str) -> bool {
        self.capabilities.contains(capability)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn node(capabilities: &[&str]) -> Node {
        Node {
            node_id: NodeId::new(1),
            name: "edge-01".to_string(),
            roles: [NodeRole::Executing, NodeRole::Operational]
                .into_iter()
                .collect(),
            capabilities: capabilities.iter().map(|c| (*c).to_string()).collect(),
            trusted: true,
        }
    }

    #[test]
    fn a_node_supports_exactly_what_it_declares() {
        let node = node(&["transport:http", "contract:json"]);
        assert!(node.supports("transport:http"));
        assert!(!node.supports("transport:mqtt"));
    }

    #[test]
    fn roles_are_a_set_in_a_fixed_order() {
        let node = node(&[]);
        assert_eq!(
            node.roles.iter().copied().collect::<Vec<_>>(),
            vec![NodeRole::Operational, NodeRole::Executing]
        );
        assert!(NodeRole::Development > NodeRole::Executing);
    }
}
