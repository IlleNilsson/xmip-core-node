//! What a node is for: its roles, the one declaration of them, and the one
//! parse of a declaration (ADR-0056, amendment 2026-10-01).
//!
//! Eight roles. Three serve one stage of the message path each — receiving,
//! processing, sending — and **executing is their sum**: one node, one
//! process, the whole Journey without a process hop, which is what buys
//! latency (the owner, 2026-10-01: *Leave Executing as a sum of Receiving,
//! Processing and Sending. Executing would be used for Low Latency*;
//! ADR-0018 clause 10a). The other four serve no stage: operational changes
//! runtime state, monitoring reads it, development is the Playground's, and
//! storage is Xmip Storage, the doorway every other node calls for all
//! storage (the owner, 2026-10-01: *to have one or more Xmip Nodes with role
//! Storage would be a safety… Xmip could just do a round robin over Xmip
//! Nodes roled Storage*; ADR-0056, amendment 2026-10-01, the Storage role).
//!
//! The words are exact lowercase only, the owner's rule of 2026-09-24 for the
//! stage words this declaration replaced: `Receiving` is an unknown word. An
//! unknown word is REFUSED, naming it and the words there are (ADR-0055);
//! it is never dropped. Nothing is ever read out of a node's name.

use serde::{Deserialize, Serialize};

use crate::Stage;

/// What a node is for. Kebab-case is the form the host plan writes; the
/// order is the order a declaration is said in.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum NodeRole {
    /// Changes runtime state or an operational outcome: claims, checkpoints,
    /// retries, resumes, suspends.
    Operational,
    /// Reads runtime state: health, figures, history, lineage.
    Monitoring,
    /// Serves the receive stage: a Receive Location starts here.
    Receiving,
    /// Serves the process stage: a Subscription routes here.
    Processing,
    /// Serves the send stage: a Send Port starts here.
    Sending,
    /// Receiving, processing and sending in one process: the low-latency
    /// role, a Journey with no process hop.
    Executing,
    /// The Playground's: exercises Xmip from outside (ADR-0028).
    Development,
    /// Xmip Storage: serves every storage operation the other nodes call,
    /// in front of the runtime and the administration databases
    /// (`deployment-model.md` sections 3 and 7).
    Storage,
}

impl NodeRole {
    /// Every role, in the order a declaration is said in.
    pub const ALL: [NodeRole; 8] = [
        NodeRole::Operational,
        NodeRole::Monitoring,
        NodeRole::Receiving,
        NodeRole::Processing,
        NodeRole::Sending,
        NodeRole::Executing,
        NodeRole::Development,
        NodeRole::Storage,
    ];

    /// The words a node may declare: each role's [`name`](Self::name).
    pub const WORDS: [&'static str; 8] = [
        "operational",
        "monitoring",
        "receiving",
        "processing",
        "sending",
        "executing",
        "development",
        "storage",
    ];

    /// The role's canonical word, as a declaration, a flag and a published
    /// record say it.
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::Operational => "operational",
            Self::Monitoring => "monitoring",
            Self::Receiving => "receiving",
            Self::Processing => "processing",
            Self::Sending => "sending",
            Self::Executing => "executing",
            Self::Development => "development",
            Self::Storage => "storage",
        }
    }

    /// The role a word names, exactly and in lowercase, or `None`.
    #[must_use]
    pub fn named(word: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|role| role.name() == word)
    }

    /// The stages of the message path the role serves, in path order:
    /// one for each of the three, all three for executing, none otherwise.
    #[must_use]
    pub const fn stages(self) -> &'static [Stage] {
        match self {
            Self::Receiving => &[Stage::Receive],
            Self::Processing => &[Stage::Process],
            Self::Sending => &[Stage::Send],
            Self::Executing => &Stage::ALL,
            Self::Operational | Self::Monitoring | Self::Development | Self::Storage => &[],
        }
    }

    /// The role that serves exactly `stage`.
    #[must_use]
    pub const fn serving(stage: Stage) -> Self {
        match stage {
            Stage::Receive => Self::Receiving,
            Stage::Process => Self::Processing,
            Stage::Send => Self::Sending,
        }
    }

    /// `roles` as one declaration says them: each at most once, in
    /// [`ALL`](Self::ALL)'s order, and the three stage roles said as
    /// executing wherever together they serve the whole path, because
    /// executing is their sum and one thing has one name.
    #[must_use]
    pub fn said(roles: &[NodeRole]) -> Vec<NodeRole> {
        let whole = Stage::ALL
            .into_iter()
            .all(|stage| roles.iter().any(|role| role.stages().contains(&stage)));
        Self::ALL
            .into_iter()
            .filter(|role| roles.contains(role) || (whole && *role == Self::Executing))
            .filter(|role| !(whole && matches!(role.stages(), [_])))
            .collect()
    }

    /// The roles a declaration names: words separated by commas or `+`,
    /// blanks ignored, returned as [`said`](Self::said) says them. The empty
    /// declaration declares no role, which is a real answer.
    ///
    /// # Errors
    ///
    /// When any word is no role: REFUSED, naming every such word and the
    /// words a node may declare (ADR-0055).
    pub fn declared(raw: &str) -> Result<Vec<Self>, String> {
        let words: Vec<&str> = raw
            .split([',', '+'])
            .map(str::trim)
            .filter(|word| !word.is_empty())
            .collect();
        let strangers: Vec<&str> = words
            .iter()
            .copied()
            .filter(|word| Self::named(word).is_none())
            .collect();
        if !strangers.is_empty() {
            return Err(format!(
                "REFUSED: no role is called {}; a node declares {}, or nothing at all.",
                strangers.join(", "),
                Self::WORDS.join(", ")
            ));
        }
        let roles: Vec<Self> = words.into_iter().filter_map(Self::named).collect();
        Ok(Self::said(&roles))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_words_are_the_roles_names_in_order() {
        assert_eq!(NodeRole::ALL.map(NodeRole::name), NodeRole::WORDS);
        assert!(NodeRole::Development > NodeRole::Executing);
        assert!(NodeRole::Storage > NodeRole::Development);
        assert!(NodeRole::Storage.stages().is_empty());
        for stage in Stage::ALL {
            assert_eq!(NodeRole::serving(stage).stages(), [stage]);
        }
    }

    #[test]
    fn executing_is_the_sum_of_the_three_and_said_as_one() {
        assert_eq!(NodeRole::Executing.stages(), Stage::ALL);
        assert_eq!(
            NodeRole::declared("sending + receiving, processing"),
            Ok(vec![NodeRole::Executing])
        );
        assert_eq!(
            NodeRole::declared("executing,receiving,monitoring"),
            Ok(vec![NodeRole::Monitoring, NodeRole::Executing])
        );
        assert_eq!(
            NodeRole::declared("processing+sending"),
            Ok(vec![NodeRole::Processing, NodeRole::Sending])
        );
        assert!(NodeRole::Operational.stages().is_empty());
    }

    #[test]
    fn a_declaration_reads_once_each_and_empty_is_none() {
        assert_eq!(
            NodeRole::declared(" sending + receiving ,, receiving"),
            Ok(vec![NodeRole::Receiving, NodeRole::Sending])
        );
        assert_eq!(NodeRole::declared(""), Ok(Vec::new()));
        assert_eq!(NodeRole::declared(" , + "), Ok(Vec::new()));
    }

    #[test]
    fn a_word_is_exact_lowercase_and_an_unknown_one_is_refused_by_name() {
        assert_eq!(
            NodeRole::declared("Sending + RECEIVING"),
            Err(
                "REFUSED: no role is called Sending, RECEIVING; a node declares operational, \
                 monitoring, receiving, processing, sending, executing, development, \
                 storage, or nothing at all."
                    .to_string()
            )
        );
        let refusal = NodeRole::declared("receiving,receive").expect_err("receive is a stage");
        assert!(refusal.contains("called receive;"), "{refusal}");
        assert!(NodeRole::named("").is_none());
    }
}
