//! What a piece of work requires of the node it is placed on (ADR-0056
//! clause 3, amendment 2026-11-13).
//!
//! Written where the work is configured, in the terms a node declares
//! itself in, so an operator reads what the work needs without running it.
//! A Receive Location requires the receive stage served and a Send Port the
//! send stage; a node whose roles serve that stage satisfies it — the
//! executing role serves every stage, so it satisfies both
//! ([`Capability::can`]). Work that must reach off the machine requires the
//! online capability too (ADR-0045). Authentication and runtime capability
//! are not modeled, as a node does not declare them
//! ([`Capability::evidence`]).

use crate::{Capability, Stage};

/// What one piece of work requires of the node that takes it.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Requirement {
    /// The stage of the message path the work needs served, if any.
    stage: Option<Stage>,
    /// Whether the work needs a route off the machine.
    online: bool,
}

impl Requirement {
    /// Work that requires nothing: every node satisfies it.
    #[must_use]
    pub const fn none() -> Self {
        Self {
            stage: None,
            online: false,
        }
    }

    /// Work that needs `stage` served: a Receive Location the receive stage,
    /// a Send Port the send stage.
    #[must_use]
    pub const fn serving(stage: Stage) -> Self {
        Self {
            stage: Some(stage),
            online: false,
        }
    }

    /// The same requirement, saying whether it needs a route off the
    /// machine.
    #[must_use]
    pub const fn with_online(mut self, online: bool) -> Self {
        self.online = online;
        self
    }

    /// The stage the work needs served, if any.
    #[must_use]
    pub const fn stage(self) -> Option<Stage> {
        self.stage
    }

    /// Whether the work needs a route off the machine.
    #[must_use]
    pub const fn is_online(self) -> bool {
        self.online
    }

    /// What it requires, in words, as a refusal names it: `serving Receive`,
    /// `serving Send, online`, `online`, or `nothing`.
    #[must_use]
    pub fn said(self) -> String {
        let serving = self.stage.map(|stage| format!("serving {}", stage.title()));
        let online = self.online.then(|| "online".to_string());
        let said: Vec<String> = serving.into_iter().chain(online).collect();
        if said.is_empty() {
            "nothing".to_string()
        } else {
            said.join(", ")
        }
    }
}

impl Capability {
    /// Whether a node declaring this satisfies `requirement`: one of its
    /// roles serves the stage the work needs, and it is online where the
    /// work needs a route off the machine (ADR-0056 clause 4: placement is
    /// a match).
    #[must_use]
    pub fn satisfies(&self, requirement: Requirement) -> bool {
        requirement.stage.is_none_or(|stage| self.can(stage))
            && (!requirement.online || self.is_online())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::NodeRole;

    #[test]
    fn a_role_serving_the_stage_satisfies_it_and_executing_serves_every_stage() {
        let receive = Requirement::serving(Stage::Receive);
        let send = Requirement::serving(Stage::Send);
        let receiving = Capability::of(&[NodeRole::Receiving]);
        let executing = Capability::of(&[NodeRole::Executing]);
        assert!(receiving.satisfies(receive));
        assert!(!receiving.satisfies(send));
        assert!(executing.satisfies(receive) && executing.satisfies(send));
        assert!(!Capability::of(&[NodeRole::Storage]).satisfies(receive));
        assert!(!Capability::none().satisfies(send));
    }

    #[test]
    fn work_needing_a_route_off_the_machine_needs_an_online_node() {
        let send = Requirement::serving(Stage::Send).with_online(true);
        let sending = Capability::of(&[NodeRole::Sending]);
        assert!(!sending.satisfies(send));
        assert!(sending.clone().with_online(true).satisfies(send));
        assert!(sending.satisfies(Requirement::serving(Stage::Send)));
        assert!(!Capability::none().satisfies(Requirement::none().with_online(true)));
    }

    #[test]
    fn nothing_required_is_satisfied_by_every_node() {
        assert!(Capability::none().satisfies(Requirement::none()));
        assert_eq!(Requirement::default(), Requirement::none());
    }

    #[test]
    fn a_requirement_is_said_in_words() {
        assert_eq!(
            Requirement::serving(Stage::Receive).said(),
            "serving Receive"
        );
        assert_eq!(
            Requirement::serving(Stage::Process)
                .with_online(true)
                .said(),
            "serving Work Process, online"
        );
        assert_eq!(Requirement::none().with_online(true).said(), "online");
        assert_eq!(Requirement::none().said(), "nothing");
        assert_eq!(Requirement::serving(Stage::Send).stage(), Some(Stage::Send));
        assert!(!Requirement::serving(Stage::Send).is_online());
    }
}
