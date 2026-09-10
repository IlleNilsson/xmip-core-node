#![forbid(unsafe_code)]

use std::collections::BTreeSet;
use xcore::NodeId;

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
