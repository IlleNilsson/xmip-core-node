#![forbid(unsafe_code)]

pub mod capability;
pub mod declaration;
pub mod requirement;
pub mod role;
pub mod scope;
pub mod stage;
pub mod standing;
pub mod unit;
pub mod wildcard;

pub use capability::Capability;
pub use declaration::{Declaration, Purpose};
pub use requirement::Requirement;
pub use role::NodeRole;
pub use scope::Scope;
pub use stage::Stage;
pub use standing::Standing;

use std::collections::BTreeSet;

use serde::{Deserialize, Serialize};
use xcore::NodeId;

/// One machine of a cluster, with the roles it carries for its purpose
/// ([`NodeRole`]; ADR-0052, ruled 2026-09-19). What it can take is what its
/// roles declare ([`Capability`]), matched against what work requires
/// ([`Requirement`]; ADR-0056).
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct Node {
    pub node_id: NodeId,
    pub name: String,
    pub roles: BTreeSet<NodeRole>,
    pub trusted: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn roles_are_a_set_in_a_fixed_order() {
        let cluster = configure::fixture::test_cluster();
        let node = Node {
            node_id: NodeId::new(1),
            name: cluster.node(0).name.clone(),
            roles: [NodeRole::Executing, NodeRole::Operational]
                .into_iter()
                .collect(),
            trusted: true,
        };
        assert_eq!(
            node.roles.iter().copied().collect::<Vec<_>>(),
            vec![NodeRole::Operational, NodeRole::Executing]
        );
    }
}
