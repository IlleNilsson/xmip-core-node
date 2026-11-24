//! What a piece of work requires of the node it is placed on (ADR-0056
//! clause 3, amendments 2026-11-13 and 2026-11-24).
//!
//! Written where the work is configured, in the terms a node declares
//! itself in, so an operator reads what the work needs without running it.
//! A Receive Location requires the receive stage served and a Send Port the
//! send stage; a node whose roles serve that stage satisfies it — the
//! executing role serves every stage, so it satisfies both
//! ([`Capability::can`]). Work may require Node capabilities as well, words a
//! node must declare for what its environment gives it — a firewall opening,
//! software installed, [`ONLINE`] for a route off the machine (ADR-0045) —
//! matched and never read ([`Capability::node_capabilities`]).
//! Authentication and runtime capability are not modeled, as a node does not
//! declare them ([`Capability::evidence`]).

use crate::{Capability, ONLINE, Stage};

/// What one piece of work requires of the node that takes it.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Requirement {
    /// The stage of the message path the work needs served, if any.
    stage: Option<Stage>,
    /// The Node capabilities the node must declare, each.
    required: Vec<String>,
}

impl Requirement {
    /// Work that requires nothing: every node satisfies it.
    #[must_use]
    pub const fn none() -> Self {
        Self {
            stage: None,
            required: Vec::new(),
        }
    }

    /// Work that needs `stage` served: a Receive Location the receive stage,
    /// a Send Port the send stage.
    #[must_use]
    pub const fn serving(stage: Stage) -> Self {
        Self {
            stage: Some(stage),
            required: Vec::new(),
        }
    }

    /// Work that reaches off the machine: the Node capability [`ONLINE`]
    /// required, matched as every other is — the one Xmip itself asks after
    /// (ADR-0045; ADR-0056, amendment 2026-11-24).
    #[must_use]
    pub fn online() -> Self {
        Self {
            stage: None,
            required: vec![ONLINE.to_string()],
        }
    }

    /// The same requirement, needing a node that declares each of the Node
    /// capabilities `words` names (ADR-0056, amendment 2026-11-24).
    ///
    /// # Errors
    ///
    /// As [`Capability::node_capabilities`] refuses them.
    pub fn requiring(mut self, words: &[String]) -> Result<Self, String> {
        self.required = Capability::node_capabilities(words)?;
        Ok(self)
    }

    /// The stage the work needs served, if any.
    #[must_use]
    pub const fn stage(&self) -> Option<Stage> {
        self.stage
    }

    /// The Node capabilities the node must declare.
    #[must_use]
    pub fn required(&self) -> &[String] {
        &self.required
    }

    /// What it requires, in words, as a refusal names it: `serving Receive`,
    /// `serving Send, online`, `serving Work Process, order-approval-module`,
    /// `online`, or `nothing`.
    #[must_use]
    pub fn said(&self) -> String {
        let serving = self.stage.map(|stage| format!("serving {}", stage.title()));
        let said: Vec<String> = serving
            .into_iter()
            .chain(self.required.iter().cloned())
            .collect();
        if said.is_empty() {
            "nothing".to_string()
        } else {
            said.join(", ")
        }
    }
}

impl Capability {
    /// Whether a node declaring this satisfies `requirement`: one of its
    /// roles serves the stage the work needs, and it declares every Node
    /// capability the work requires (ADR-0056 clause 4: placement is a
    /// match).
    #[must_use]
    pub fn satisfies(&self, requirement: &Requirement) -> bool {
        requirement.stage.is_none_or(|stage| self.can(stage))
            && requirement
                .required
                .iter()
                .all(|word| self.declared().contains(word))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::NodeRole;

    fn words(said: &[&str]) -> Vec<String> {
        said.iter().map(ToString::to_string).collect()
    }

    #[test]
    fn a_role_serving_the_stage_satisfies_it_and_executing_serves_every_stage() {
        let receive = Requirement::serving(Stage::Receive);
        let send = Requirement::serving(Stage::Send);
        let receiving = Capability::of(&[NodeRole::Receiving]);
        let executing = Capability::of(&[NodeRole::Executing]);
        assert!(receiving.satisfies(&receive));
        assert!(!receiving.satisfies(&send));
        assert!(executing.satisfies(&receive) && executing.satisfies(&send));
        assert!(!Capability::of(&[NodeRole::Storage]).satisfies(&receive));
        assert!(!Capability::none().satisfies(&send));
    }

    /// Online is a Node capability like any other: a node declares it, and
    /// work reaching off the machine requires it; one that declares nothing
    /// is offline (ADR-0056, amendment 2026-11-24).
    #[test]
    fn work_needing_a_route_off_the_machine_needs_a_node_declaring_online() {
        let send = Requirement::serving(Stage::Send)
            .requiring(&words(&[ONLINE]))
            .expect("a word");
        let sending = Capability::of(&[NodeRole::Sending]);
        assert!(!sending.satisfies(&send));
        let online = sending
            .clone()
            .with_declared(&words(&[ONLINE]))
            .expect("a word");
        assert!(online.satisfies(&send));
        assert!(online.satisfies(&Requirement::online()));
        assert!(sending.satisfies(&Requirement::serving(Stage::Send)));
        assert!(!Capability::none().satisfies(&Requirement::online()));
    }

    #[test]
    fn work_requiring_node_capabilities_needs_a_node_declaring_every_one() {
        let approval = Requirement::serving(Stage::Process)
            .requiring(&words(&["order-approval-module"]))
            .expect("a word");
        let processing = Capability::of(&[NodeRole::Processing]);
        assert!(!processing.satisfies(&approval), "it declares none");
        let installed = processing
            .clone()
            .with_declared(&words(&["erp-direct", "order-approval-module"]))
            .expect("two words");
        assert!(installed.satisfies(&approval));
        assert!(installed.satisfies(&Requirement::serving(Stage::Process)));
        let both = Requirement::serving(Stage::Process)
            .requiring(&words(&["order-approval-module", "firewall-party-e"]))
            .expect("two words");
        assert!(!installed.satisfies(&both), "one of two is not every one");
        let sending = Capability::of(&[NodeRole::Sending])
            .with_declared(&words(&["order-approval-module"]))
            .expect("a word");
        assert!(
            !sending.satisfies(&approval),
            "a word never stands for a role"
        );
        assert!(
            Requirement::none()
                .requiring(&words(&["Order approval"]))
                .is_err()
        );
    }

    #[test]
    fn nothing_required_is_satisfied_by_every_node() {
        assert!(Capability::none().satisfies(&Requirement::none()));
        assert_eq!(Requirement::default(), Requirement::none());
    }

    #[test]
    fn a_requirement_is_said_in_words() {
        assert_eq!(
            Requirement::serving(Stage::Receive).said(),
            "serving Receive"
        );
        assert_eq!(
            Requirement::serving(Stage::Send)
                .requiring(&words(&[ONLINE]))
                .expect("a word")
                .said(),
            "serving Send, online"
        );
        assert_eq!(
            Requirement::serving(Stage::Process)
                .requiring(&words(&["order-approval-module"]))
                .expect("a word")
                .said(),
            "serving Work Process, order-approval-module"
        );
        assert_eq!(Requirement::online().said(), "online");
        assert_eq!(Requirement::none().said(), "nothing");
        assert_eq!(Requirement::serving(Stage::Send).stage(), Some(Stage::Send));
        assert_eq!(
            Requirement::serving(Stage::Send).required(),
            [] as [String; 0]
        );
    }
}
