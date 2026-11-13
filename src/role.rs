//! What a node is for: its roles, the one declaration of them, and the one
//! parse of a declaration (ADR-0056, amendment 2026-10-01).
//!
//! Eight roles. Three serve one stage of the message path each — receiving,
//! processing, sending — and **executing is their sum**: one node, one
//! process, a Message's whole Journey without a process hop, which is what buys
//! latency (the owner, 2026-10-01: *Leave Executing as a sum of Receiving,
//! Processing and Sending. Executing would be used for Low Latency*;
//! ADR-0018 clause 10a). The other four serve no stage: operational changes
//! runtime state, monitoring reads it, development is the Essentials', and
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
    /// role, a Message with no process hop.
    Executing,
    /// The Essentials': exercises Xmip from outside (ADR-0028).
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

    /// What a node declaring the role is for, in the words every surface
    /// shows a node's purpose in — on hover over it on the Topology and the
    /// Monitor, Web and Desktop alike, and beside it in the command line's
    /// and PowerShell's node rows (the owner, 2026-11-12: *There has to be a
    /// tooltip, regardless of Web or MAUI. what are nodes purpose*). Worded
    /// from `deployment-model.md` section 3's *May* column; said here once.
    #[must_use]
    pub const fn meaning(self) -> &'static str {
        match self {
            Self::Operational => "changes runtime state or an operational outcome",
            Self::Monitoring => "inspects runtime state: health, figures, history, lineage",
            Self::Receiving => "takes Streams in and makes Messages",
            Self::Processing => "routes and transforms Messages by Subscription",
            Self::Sending => "delivers Messages out",
            Self::Executing => "receives, processes and sends in one process",
            Self::Development => "exercises Xmip from outside",
            Self::Storage => "is the doorway to all storage: Xmip Storage",
        }
    }

    /// What a node declaring `roles` is for: each role said as
    /// [`said`](Self::said) says them, with its [`meaning`](Self::meaning),
    /// one per line; a node declaring none says so, since declaring none is
    /// a choice, not an absence. The one purpose every surface shows
    /// (`xmip_role_purpose_v1`, `xmip_operate.h` section 7).
    #[must_use]
    pub fn purpose(roles: &[NodeRole]) -> String {
        let said = Self::said(roles);
        if said.is_empty() {
            return "declares no role".to_string();
        }
        said.iter()
            .map(|role| format!("{}: {}", role.name(), role.meaning()))
            .collect::<Vec<_>>()
            .join("\n")
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
        assert_eq!(NodeRole::Storage.stages(), []);
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
        assert_eq!(NodeRole::Operational.stages(), []);
    }

    #[test]
    fn every_role_has_a_meaning_and_a_purpose_says_each_once_in_order() {
        for role in NodeRole::ALL {
            assert!(!role.meaning().is_empty(), "{}", role.name());
        }
        assert_eq!(
            NodeRole::purpose(&[NodeRole::Monitoring, NodeRole::Operational]),
            "operational: changes runtime state or an operational outcome\n\
             monitoring: inspects runtime state: health, figures, history, lineage"
        );
        assert_eq!(
            NodeRole::purpose(&[NodeRole::Sending, NodeRole::Receiving, NodeRole::Processing]),
            "executing: receives, processes and sends in one process",
            "the three said as their sum"
        );
        assert_eq!(NodeRole::purpose(&[]), "declares no role");
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
