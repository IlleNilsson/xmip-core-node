//! What a node declares it can do, and the two forms it says it in
//! (ADR-0056).
//!
//! ADR-0056: *a node declares its capabilities, and work is placed on a node
//! whose capabilities satisfy what the work requires.* A name is not a
//! criterion. Two of ADR-0056's four kinds are modelled:
//!
//!   - **Feature capability** — which [`Stage`]s of the message path the node
//!     can serve. A node may declare more than one; a node that declares none
//!     runs whole tests itself.
//!   - **Online capability** — whether a route off this machine may be
//!     assumed (ADR-0045).
//!
//! Authentication and runtime capability are not modelled, and the evidence a
//! node publishes says so rather than staying silent.
//!
//! A declaration is said in two forms, each written and read here and
//! nowhere else (open problem 25): the **evidence** a node publishes about
//! itself — `declares receive,send; online; …` — and the **entry** a run
//! lists it by — `edge-01=receive+send`, or the bare name for a node that
//! declares no stage. The runtime's library forwards both readings to the
//! surfaces (`xmip_capability_published_v1`, `xmip_capability_entry_v1`,
//! `xmip_operate.h` section 7), so `Xmip.Surface` keeps no parse of its own.
//! Where the declaration is placed is not decided here (open problem 25,
//! row o).

use crate::Stage;

/// What a node that declares no stage publishes in place of the words.
const NO_STAGE: &str = "no stage of the message path";

/// What the evidence of a declaration starts with.
const DECLARES: &str = "declares ";

/// What one node declared it can do.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Capability {
    /// Online capability: a route off this machine may be assumed.
    online: bool,
    /// Feature capability: the stages of the message path this node serves,
    /// in message-path order, each at most once.
    features: Vec<Stage>,
}

impl Capability {
    /// A node that declares nothing: no stage of the message path, offline.
    #[must_use]
    pub const fn none() -> Self {
        Self {
            online: false,
            features: Vec::new(),
        }
    }

    /// The capability declaring these stages, in message-path order.
    #[must_use]
    pub fn of(stages: &[Stage]) -> Self {
        Self {
            online: false,
            features: Stage::ALL
                .into_iter()
                .filter(|stage| stages.contains(stage))
                .collect(),
        }
    }

    /// The capability a declaration names, read by [`Stage::declared`]: the
    /// stage words, separated by commas or by `+`, lowercase exactly. Empty
    /// declares nothing.
    ///
    /// # Errors
    ///
    /// When a word is no capability: REFUSED, naming the word and the values
    /// it would take (ADR-0055).
    pub fn parse(raw: &str) -> Result<Self, String> {
        Stage::declared(raw).map(|stages| Self::of(&stages))
    }

    /// The same capability with its online capability said (ADR-0045).
    #[must_use]
    pub fn with_online(mut self, online: bool) -> Self {
        self.online = online;
        self
    }

    /// Whether a route off this machine may be assumed.
    #[must_use]
    pub const fn is_online(&self) -> bool {
        self.online
    }

    /// The stages of the message path this node serves, in path order.
    #[must_use]
    pub fn features(&self) -> &[Stage] {
        &self.features
    }

    /// Whether this node can serve `stage`.
    #[must_use]
    pub fn can(&self, stage: Stage) -> bool {
        self.features.contains(&stage)
    }

    /// Whether this node declares no stage of the message path — it runs
    /// whole tests itself, as every node did before capabilities.
    #[must_use]
    pub fn declares_no_stage(&self) -> bool {
        self.features.is_empty()
    }

    /// The feature capability as a declaration: `receive,process`, or the
    /// empty string when none is declared.
    #[must_use]
    pub fn words(&self) -> String {
        self.joined(",")
    }

    /// The word a health record carries for the online capability.
    #[must_use]
    pub const fn word(&self) -> &'static str {
        if self.online { "online" } else { "offline" }
    }

    /// What the node's own capability record says, so a surface reads a
    /// node's capabilities from the snapshot and never from its name.
    #[must_use]
    pub fn evidence(&self) -> String {
        let declared = if self.declares_no_stage() {
            NO_STAGE.to_string()
        } else {
            self.words()
        };
        format!(
            "{DECLARES}{declared}; {}; authentication and runtime capability \
             are not modelled in this rig",
            self.word()
        )
    }

    /// The capability an evidence line says, read through the same parse as
    /// a declaration. A line that is no declaration at all declares nothing,
    /// offline unless it says `; online;`.
    ///
    /// # Errors
    ///
    /// When the declaration names a word that is no capability: REFUSED, as
    /// [`Stage::declared`] says it — never read as the words that were known.
    pub fn from_evidence(evidence: &str) -> Result<Self, String> {
        let said = evidence
            .strip_prefix(DECLARES)
            .and_then(|rest| rest.split(';').next())
            .unwrap_or_default();
        let stages = if said == NO_STAGE {
            Vec::new()
        } else {
            Stage::declared(said)?
        };
        Ok(Self::of(&stages).with_online(evidence.contains("; online;")))
    }

    /// The node as a run lists it: `edge-01=receive+send`, or the bare name
    /// when it declares no stage. It says nothing of the online capability,
    /// which a run lists apart.
    #[must_use]
    pub fn entry(&self, name: &str) -> String {
        if self.declares_no_stage() {
            name.to_string()
        } else {
            format!("{name}={}", self.joined("+"))
        }
    }

    /// An entry read back: the name before the first `=`, trimmed, and the
    /// capability the rest declares — or, when it names a word that is no
    /// capability, the refusal [`Capability::parse`] gives. Whatever the node
    /// is called is read as a name and nothing else, refused or not.
    pub fn from_entry(entry: &str) -> (&str, Result<Self, String>) {
        let (name, declared) = entry.split_once('=').unwrap_or((entry, ""));
        (name.trim(), Self::parse(declared))
    }

    fn joined(&self, separator: &str) -> String {
        self.features
            .iter()
            .map(|stage| stage.name())
            .collect::<Vec<&str>>()
            .join(separator)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_node_declares_stages_by_name_and_an_unknown_word_is_refused() {
        let one = Capability::parse("receive").expect("receive is a capability");
        assert_eq!(one.features(), [Stage::Receive]);
        assert!(one.can(Stage::Receive) && !one.can(Stage::Send));
        assert!(!one.declares_no_stage());

        let two = Capability::parse(" send + receive ,, ").expect("both, in path order");
        assert_eq!(two.features(), [Stage::Receive, Stage::Send]);
        assert_eq!(two.words(), "receive,send");
        let cased = Capability::parse("Send + RECEIVE").expect_err("lowercase only");
        assert!(cased.contains("called Send, RECEIVE;"), "{cased}");

        assert_eq!(Capability::parse(""), Ok(Capability::none()));
        assert!(Capability::none().declares_no_stage());
        assert_eq!(Capability::none().words(), "");

        let refusal = Capability::parse("receive,relay").expect_err("relay is no capability");
        assert!(refusal.starts_with("REFUSED"), "{refusal}");
        assert!(
            refusal.contains("relay") && refusal.contains("process"),
            "{refusal}"
        );
    }

    #[test]
    fn the_online_capability_rides_along() {
        let capability = Capability::of(&[Stage::Process]).with_online(true);
        assert!(capability.is_online());
        assert_eq!(capability.word(), "online");
        assert_eq!(Capability::none().word(), "offline");
    }

    #[test]
    fn what_a_node_publishes_reads_back_as_what_it_declared() {
        for capability in [
            Capability::none(),
            Capability::of(&[Stage::Receive]).with_online(true),
            Capability::of(&[Stage::Receive, Stage::Process, Stage::Send]),
        ] {
            let evidence = capability.evidence();
            assert_eq!(
                Capability::from_evidence(&evidence),
                Ok(capability),
                "{evidence}"
            );
        }
        assert!(
            Capability::of(&[Stage::Send])
                .evidence()
                .contains("not modelled in this rig"),
            "the two kinds left out are said, not silent"
        );
        assert_eq!(
            Capability::none().evidence(),
            "declares no stage of the message path; offline; authentication and runtime \
             capability are not modelled in this rig"
        );
        assert_eq!(Capability::from_evidence("alive"), Ok(Capability::none()));
    }

    #[test]
    fn a_published_declaration_reads_by_the_same_rule_as_the_flag() {
        assert_eq!(
            Capability::from_evidence("declares receive,send; offline; x"),
            Ok(Capability::of(&[Stage::Receive, Stage::Send]))
        );
        let cased = Capability::from_evidence("declares Receive,SEND; offline; x")
            .expect_err("lowercase only");
        assert!(cased.contains("called Receive, SEND;"), "{cased}");
        let refusal = Capability::from_evidence("declares receive,relay; online; x")
            .expect_err("relay is no capability");
        assert!(
            refusal.starts_with("REFUSED") && refusal.contains("relay"),
            "{refusal}"
        );
    }

    #[test]
    fn an_entry_names_the_node_and_what_it_was_started_with() {
        let both = Capability::of(&[Stage::Send, Stage::Receive]);
        assert_eq!(both.entry("edge-01"), "edge-01=receive+send");
        assert_eq!(Capability::none().entry("edge-02"), "edge-02");
        assert_eq!(
            Capability::from_entry(" edge-01 =receive+send"),
            ("edge-01", Ok(both))
        );
        assert_eq!(
            Capability::from_entry("edge-02"),
            ("edge-02", Ok(Capability::none()))
        );
        let (name, refused) = Capability::from_entry("edge-03=relay");
        assert_eq!(name, "edge-03");
        assert!(refused.expect_err("relay").starts_with("REFUSED"));
    }
}
