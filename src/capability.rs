//! What a node declares, and the two forms it says it in (ADR-0056).
//!
//! ADR-0056: *a node declares its capabilities, and work is placed on a node
//! whose capabilities satisfy what the work requires.* A name is not a
//! criterion. Two things are declared here:
//!
//!   - **Roles** — the node's [`NodeRole`]s, the one declaration of what it
//!     is for. Receiving, processing and sending each serve one stage of the
//!     message path, executing serves all three in one process, and the
//!     stages a node serves are read from its roles and nowhere else
//!     (amendment 2026-10-01: the stage words a node used to declare were the
//!     same thing under a second name, and are gone). A node that declares no
//!     stage-serving role runs whole tests itself.
//!   - **Node capabilities** — words naming what the node's environment
//!     gives it, a firewall opening, software installed, a system it
//!     reaches ([`Capability::node_capabilities`]). A node's roles are
//!     Xmip's; its Node capabilities are named by whoever runs it. Xmip
//!     reads no meaning from one: it only matches the words a node declares
//!     against those its work requires (amendment 2026-11-24). One Xmip
//!     asks after itself, [`ONLINE`], a route off the machine (ADR-0045);
//!     a node declaring nothing is offline. They are the cluster's file's,
//!     and a node publishes them with its roles.
//!
//! Authentication and runtime capability are not modeled, and the evidence a
//! node publishes says so rather than staying silent.
//!
//! A declaration is said in two forms, each written and read here and
//! nowhere else (open problem 25): the **evidence** a node publishes about
//! itself — `declares receiving,sending; Node capabilities online; …` — and
//! the **entry** a run lists it by — `<name>=receiving+sending`, or the bare
//! name for a node that declares no role, its Node capabilities an entry of
//! their own, `<name>=online`. The runtime's library forwards both readings
//! to the surfaces (`xmip_capability_published_v1`,
//! `xmip_capability_entry_v1`, `xmip_operate.h` section 7), so
//! `Xmip.Surface` keeps no parse of its own.

use crate::{NodeRole, Stage};

/// What a node that declares no role publishes in place of the words.
const NO_ROLE: &str = "no role";

/// What the evidence of a declaration starts with.
const DECLARES: &str = "declares ";

/// What the evidence says a node's Node capabilities after.
const NODE_CAPABILITIES: &str = "Node capabilities ";

/// What the evidence says of a node declaring no Node capability.
const NO_NODE_CAPABILITY: &str = "no Node capability";

/// The one Node capability Xmip itself asks after: a route off the machine,
/// the internet (ADR-0045; ADR-0056, amendment 2026-11-24: *An obvious node
/// capability is online, internet access*). A node declares it as any other,
/// and one that declares nothing is offline. Every other Node capability is
/// whoever runs the node's.
pub const ONLINE: &str = "online";

/// What one node declared.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Capability {
    /// The node's roles, as [`NodeRole::said`] says them.
    roles: Vec<NodeRole>,
    /// Its Node capabilities, in the order declared.
    declared: Vec<String>,
}

impl Capability {
    /// A node that declares nothing: no role, no Node capability, offline.
    #[must_use]
    pub const fn none() -> Self {
        Self {
            roles: Vec::new(),
            declared: Vec::new(),
        }
    }

    /// The node declaring these roles, said as [`NodeRole::said`] says them.
    #[must_use]
    pub fn of(roles: &[NodeRole]) -> Self {
        Self {
            roles: NodeRole::said(roles),
            declared: Vec::new(),
        }
    }

    /// `words` read as Node capabilities, as a node declares them or work
    /// requires them: each a word of lowercase letters, digits and hyphens,
    /// opening and closing with a letter or a digit, none twice (ADR-0056,
    /// amendment 2026-11-24). Xmip reads no meaning from one.
    ///
    /// # Errors
    ///
    /// REFUSED, naming the word that is none, or the word named twice.
    pub fn node_capabilities(words: &[String]) -> Result<Vec<String>, String> {
        let mut read: Vec<String> = Vec::with_capacity(words.len());
        for word in words {
            let inner = word
                .bytes()
                .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'-');
            let end = |byte: Option<u8>| byte.is_some_and(|byte| byte != b'-');
            if !(inner && end(word.bytes().next()) && end(word.bytes().last())) {
                return Err(format!(
                    "REFUSED: '{word}' is no Node capability: a Node capability is a word of \
                     lowercase letters, digits and hyphens, opening and closing with a letter or \
                     a digit"
                ));
            }
            if read.contains(word) {
                return Err(format!(
                    "REFUSED: the Node capability '{word}' is named twice"
                ));
            }
            read.push(word.clone());
        }
        Ok(read)
    }

    /// The same declaration with its Node capabilities
    /// ([`Capability::node_capabilities`]).
    ///
    /// # Errors
    ///
    /// As [`Capability::node_capabilities`] refuses them.
    pub fn with_declared(mut self, words: &[String]) -> Result<Self, String> {
        self.declared = Self::node_capabilities(words)?;
        Ok(self)
    }

    /// Its Node capabilities, in the order declared.
    #[must_use]
    pub fn declared(&self) -> &[String] {
        &self.declared
    }

    /// The node serving these stages: the role serving each, said as one
    /// declaration — all three are executing.
    #[must_use]
    pub fn serving(stages: &[Stage]) -> Self {
        let roles: Vec<NodeRole> = stages.iter().copied().map(NodeRole::serving).collect();
        Self::of(&roles)
    }

    /// The declaration a text names, read by [`NodeRole::declared`]: role
    /// words separated by commas or by `+`, lowercase exactly. Empty
    /// declares nothing.
    ///
    /// # Errors
    ///
    /// When a word is no role: REFUSED, naming the word and the words there
    /// are (ADR-0055).
    pub fn parse(raw: &str) -> Result<Self, String> {
        NodeRole::declared(raw).map(|roles| Self::of(&roles))
    }

    /// The node's roles, in declaration order.
    #[must_use]
    pub fn roles(&self) -> &[NodeRole] {
        &self.roles
    }

    /// The stages of the message path the node's roles serve, in path order.
    #[must_use]
    pub fn stages(&self) -> Vec<Stage> {
        Stage::ALL
            .into_iter()
            .filter(|stage| self.can(*stage))
            .collect()
    }

    /// Whether one of the node's roles serves `stage`.
    #[must_use]
    pub fn can(&self, stage: Stage) -> bool {
        self.roles.iter().any(|role| role.stages().contains(&stage))
    }

    /// Whether the node carries a Message its whole Journey in one process: the
    /// executing role (ADR-0018 clause 10a).
    #[must_use]
    pub fn executes(&self) -> bool {
        self.roles.contains(&NodeRole::Executing)
    }

    /// Whether the node carries an operator's act to another node — writes
    /// it to Xmip Storage, asked of the node that owns what it acts on —
    /// the operational role, its door for operator acts (ADR-0056 and
    /// ADR-0014, amendments 2026-11-13). Every node takes the acts asked of
    /// itself, whatever it declares.
    #[must_use]
    pub fn carries_acts(&self) -> bool {
        self.roles.contains(&NodeRole::Operational)
    }

    /// Whether the node gathers its cluster's view — follows every member's
    /// publication over its operate listener as an Observer, and answers
    /// the cluster's — and is listed an Observer on every node: the
    /// monitoring role (ADR-0027 decision 8, ADR-0056 and ADR-0009,
    /// amendments 2026-11-13).
    #[must_use]
    pub fn gathers(&self) -> bool {
        self.roles.contains(&NodeRole::Monitoring)
    }

    /// Whether no role of the node serves a stage of the message path — it
    /// runs whole tests itself, as every node did before capabilities.
    #[must_use]
    pub fn declares_no_stage(&self) -> bool {
        Stage::ALL.into_iter().all(|stage| !self.can(stage))
    }

    /// The roles as a declaration: `receiving,processing`, or the empty
    /// string when none is declared.
    #[must_use]
    pub fn words(&self) -> String {
        self.joined(",")
    }

    /// Its Node capabilities as a record says them: `Node capabilities
    /// online, erp-direct`, or `no Node capability`.
    #[must_use]
    pub fn named(&self) -> String {
        if self.declared.is_empty() {
            NO_NODE_CAPABILITY.to_string()
        } else {
            format!("{NODE_CAPABILITIES}{}", self.declared.join(", "))
        }
    }

    /// What the node's own capability record says, so a surface reads a
    /// node's roles and Node capabilities from the snapshot and never from
    /// its name.
    #[must_use]
    pub fn evidence(&self) -> String {
        let declared = if self.roles.is_empty() {
            NO_ROLE.to_string()
        } else {
            self.words()
        };
        format!(
            "{DECLARES}{declared}; {}; authentication and runtime capability \
             are not declared in this record",
            self.named()
        )
    }

    /// The declaration an evidence line says, read through the same parses
    /// as a declaration. A line that is no declaration at all declares
    /// nothing, and one that names no Node capability none.
    ///
    /// # Errors
    ///
    /// When the declaration names a word that is no role, or a Node
    /// capability that is none: REFUSED, as [`NodeRole::declared`] and
    /// [`Capability::node_capabilities`] say it — never read as the words
    /// that were known.
    pub fn from_evidence(evidence: &str) -> Result<Self, String> {
        let mut said = evidence
            .strip_prefix(DECLARES)
            .map(|rest| rest.split(';').map(str::trim))
            .into_iter()
            .flatten();
        let roles = match said.next().unwrap_or_default() {
            NO_ROLE | "" => Vec::new(),
            words => NodeRole::declared(words)?,
        };
        let declared: Vec<String> = Self::named_in(evidence)
            .into_iter()
            .map(str::to_string)
            .collect();
        Self::of(&roles).with_declared(&declared)
    }

    /// The Node capabilities an evidence line says, as it writes them and
    /// unchecked: what [`Capability::from_evidence`] reads, and what a
    /// surface is handed borrowed from the line (`xmip_operate.h` section
    /// 7). None where it says none, or is no declaration.
    #[must_use]
    pub fn named_in(evidence: &str) -> Vec<&str> {
        evidence
            .strip_prefix(DECLARES)
            .and_then(|rest| rest.split(';').nth(1))
            .map(str::trim)
            .and_then(|named| named.strip_prefix(NODE_CAPABILITIES))
            .map(|words| words.split(", ").collect())
            .unwrap_or_default()
    }

    /// The node as a run lists it: `<name>=receiving+sending`, or the bare
    /// name when it declares no role. It says nothing of its Node
    /// capabilities, which a run lists apart ([`Capability::declared_entry`]).
    #[must_use]
    pub fn entry(&self, name: &str) -> String {
        if self.roles.is_empty() {
            name.to_string()
        } else {
            format!("{name}={}", self.joined("+"))
        }
    }

    /// An entry read back: the name before the first `=`, trimmed, and the
    /// declaration the rest says — or, when it names a word that is no role,
    /// the refusal [`Capability::parse`] gives. Whatever the node is called
    /// is read as a name and nothing else, refused or not.
    pub fn from_entry(entry: &str) -> (&str, Result<Self, String>) {
        let (name, declared) = entry.split_once('=').unwrap_or((entry, ""));
        (name.trim(), Self::parse(declared))
    }

    /// The node's Node capabilities as a run lists them beside its roles:
    /// `<name>=online+erp-direct`, or the bare name where it declares none.
    #[must_use]
    pub fn declared_entry(&self, name: &str) -> String {
        if self.declared.is_empty() {
            name.to_string()
        } else {
            format!("{name}={}", self.declared.join("+"))
        }
    }

    /// A [`Capability::declared_entry`] read back: the name before the
    /// first `=`, trimmed, and the Node capabilities the rest says, by `+`,
    /// read as [`Capability::node_capabilities`] reads them.
    pub fn from_declared_entry(entry: &str) -> (&str, Result<Vec<String>, String>) {
        let (name, said) = entry.split_once('=').unwrap_or((entry, ""));
        let words: Vec<String> = said
            .split('+')
            .map(str::trim)
            .filter(|word| !word.is_empty())
            .map(str::to_string)
            .collect();
        (name.trim(), Self::node_capabilities(&words))
    }

    fn joined(&self, separator: &str) -> String {
        self.roles
            .iter()
            .map(|role| role.name())
            .collect::<Vec<&str>>()
            .join(separator)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_node_declares_roles_by_name_and_an_unknown_word_is_refused() {
        let one = Capability::parse("receiving").expect("receiving is a role");
        assert_eq!(one.roles(), [NodeRole::Receiving]);
        assert_eq!(one.stages(), [Stage::Receive]);
        assert!(one.can(Stage::Receive) && !one.can(Stage::Send));
        assert!(!one.declares_no_stage() && !one.executes());

        let two = Capability::parse(" sending + receiving ,, ").expect("both");
        assert_eq!(two.roles(), [NodeRole::Receiving, NodeRole::Sending]);
        assert_eq!(two.words(), "receiving,sending");
        let cased = Capability::parse("Sending + RECEIVING").expect_err("lowercase only");
        assert!(cased.contains("called Sending, RECEIVING;"), "{cased}");

        assert_eq!(Capability::parse(""), Ok(Capability::none()));
        assert!(Capability::none().declares_no_stage());
        assert_eq!(Capability::none().words(), "");

        let refusal = Capability::parse("receiving,relay").expect_err("relay is no role");
        assert!(refusal.starts_with("REFUSED"), "{refusal}");
        assert!(refusal.contains("relay") && refusal.contains("processing"));
    }

    #[test]
    fn executing_serves_the_whole_path_and_a_role_off_the_path_serves_none() {
        let whole = Capability::parse("receiving+processing+sending").expect("the three");
        assert_eq!(whole.roles(), [NodeRole::Executing]);
        assert_eq!(whole.stages(), Stage::ALL);
        assert!(whole.executes());
        assert_eq!(Capability::serving(&Stage::ALL), whole);
        assert_eq!(
            Capability::serving(&[Stage::Send, Stage::Receive]).words(),
            "receiving,sending"
        );

        let watching = Capability::parse("monitoring,operational").expect("two roles");
        assert_eq!(watching.words(), "operational,monitoring");
        assert!(watching.declares_no_stage());
    }

    #[test]
    fn only_the_operational_role_carries_an_act_to_another_node() {
        assert!(Capability::of(&[NodeRole::Operational]).carries_acts());
        let edge = Capability::parse("executing,monitoring,operational").expect("an edge node");
        assert!(edge.carries_acts());
        for alone in NodeRole::ALL
            .into_iter()
            .filter(|role| *role != NodeRole::Operational)
        {
            assert!(!Capability::of(&[alone]).carries_acts(), "{alone:?}");
        }
        assert!(!Capability::none().carries_acts());
    }

    #[test]
    fn only_the_monitoring_role_gathers_the_cluster() {
        assert!(Capability::of(&[NodeRole::Monitoring]).gathers());
        let edge = Capability::parse("executing,monitoring,operational").expect("an edge node");
        assert!(edge.gathers());
        for alone in NodeRole::ALL
            .into_iter()
            .filter(|role| *role != NodeRole::Monitoring)
        {
            assert!(!Capability::of(&[alone]).gathers(), "{alone:?}");
        }
        assert!(!Capability::none().gathers());
    }

    /// Online is a Node capability like any other, and a node declaring
    /// nothing is offline (ADR-0056, amendment 2026-11-24).
    #[test]
    fn online_is_a_node_capability_and_nothing_declared_is_offline() {
        let online = Capability::of(&[NodeRole::Processing])
            .with_declared(&[ONLINE.to_string()])
            .expect("a word");
        assert_eq!(online.declared(), [ONLINE]);
        assert_eq!(online.named(), "Node capabilities online");
        assert_eq!(Capability::none().named(), "no Node capability");
        assert_eq!(
            Capability::node_capabilities(&[ONLINE.to_string()]),
            Ok(vec![ONLINE.to_string()])
        );
    }

    #[test]
    fn node_capabilities_are_words_each_once_and_anything_else_is_refused() {
        let words = |said: &[&str]| said.iter().map(ToString::to_string).collect::<Vec<_>>();
        let declared = Capability::of(&[NodeRole::Processing])
            .with_declared(&words(&["order-approval-module", "erp-direct", "tls13"]))
            .expect("three words");
        assert_eq!(
            declared.declared(),
            words(&["order-approval-module", "erp-direct", "tls13"])
        );
        assert_eq!(Capability::none().declared(), [] as [String; 0]);
        for wrong in [
            "Firewall",
            "erp direct",
            "-erp",
            "erp-",
            "",
            "erp_direct",
            "érp",
        ] {
            let refused = Capability::node_capabilities(&words(&[wrong])).expect_err(wrong);
            assert!(
                refused.starts_with(&format!("REFUSED: '{wrong}' is no Node capability")),
                "{refused}"
            );
        }
        let twice = Capability::node_capabilities(&words(&["erp-direct", "erp-direct"]));
        assert_eq!(
            twice,
            Err("REFUSED: the Node capability 'erp-direct' is named twice".to_string())
        );
    }

    #[test]
    fn what_a_node_publishes_reads_back_as_what_it_declared() {
        let receiving = Capability::of(&[NodeRole::Receiving])
            .with_declared(&[ONLINE.to_string(), "firewall-party-x".to_string()])
            .expect("two words");
        for capability in [
            Capability::none(),
            receiving.clone(),
            Capability::of(&[NodeRole::Executing, NodeRole::Monitoring]),
        ] {
            let evidence = capability.evidence();
            assert_eq!(
                Capability::from_evidence(&evidence),
                Ok(capability),
                "{evidence}"
            );
        }
        assert_eq!(
            receiving.evidence(),
            "declares receiving; Node capabilities online, firewall-party-x; authentication \
             and runtime capability are not declared in this record"
        );
        assert_eq!(
            Capability::named_in(&receiving.evidence()),
            [ONLINE, "firewall-party-x"]
        );
        assert!(
            Capability::of(&[NodeRole::Sending])
                .evidence()
                .contains("not declared in this record"),
            "the two kinds left out are said, not silent"
        );
        assert_eq!(
            Capability::none().evidence(),
            "declares no role; no Node capability; authentication and runtime capability are \
             not declared in this record"
        );
        assert_eq!(Capability::from_evidence("alive"), Ok(Capability::none()));
        assert_eq!(Capability::named_in("alive"), [] as [&str; 0]);
    }

    #[test]
    fn a_published_declaration_reads_by_the_same_rule_as_the_flag() {
        assert_eq!(
            Capability::from_evidence("declares receiving,sending; no Node capability; x"),
            Ok(Capability::of(&[NodeRole::Receiving, NodeRole::Sending]))
        );
        let cased = Capability::from_evidence("declares Receiving,SENDING; no Node capability; x")
            .expect_err("lowercase only");
        assert!(cased.contains("called Receiving, SENDING;"), "{cased}");
        let upper = Capability::from_evidence("declares receiving; Node capabilities Online; x")
            .expect_err("a Node capability is lowercase");
        assert!(upper.contains("'Online' is no Node capability"), "{upper}");
        let refusal = Capability::from_evidence("declares receive,send; no Node capability; x")
            .expect_err("a stage word is no role");
        assert!(
            refusal.starts_with("REFUSED") && refusal.contains("receive, send"),
            "{refusal}"
        );
    }

    #[test]
    fn a_run_lists_a_node_s_node_capabilities_as_an_entry_of_its_own() {
        let cluster = configure::fixture::test_cluster();
        let [one, two] = [0, 1].map(|place| cluster.node(place).name.as_str());
        let words = [ONLINE.to_string(), "erp-direct".to_string()];
        let declared = Capability::none().with_declared(&words).expect("two words");
        assert_eq!(
            declared.declared_entry(one),
            format!("{one}=online+erp-direct")
        );
        assert_eq!(Capability::none().declared_entry(two), two);
        let said = format!(" {one} =online+erp-direct");
        let (name, read) = Capability::from_declared_entry(&said);
        assert_eq!((name, read), (one, Ok(words.to_vec())));
        assert_eq!(Capability::from_declared_entry(two), (two, Ok(Vec::new())));
        let (_, refused) = Capability::from_declared_entry(&format!("{one}=Online"));
        assert!(refused.expect_err("lowercase").starts_with("REFUSED"));
    }

    #[test]
    fn an_entry_names_the_node_and_what_it_was_started_with() {
        let cluster = configure::fixture::test_cluster();
        let [one, two, three] = [0, 1, 2].map(|place| cluster.node(place).name.as_str());
        let both = Capability::of(&[NodeRole::Sending, NodeRole::Receiving]);
        assert_eq!(both.entry(one), format!("{one}=receiving+sending"));
        assert_eq!(Capability::none().entry(two), two);
        assert_eq!(
            Capability::from_entry(&format!(" {one} =receiving+sending")),
            (one, Ok(both))
        );
        assert_eq!(Capability::from_entry(two), (two, Ok(Capability::none())));
        let relay = format!("{three}=relay");
        let (name, refused) = Capability::from_entry(&relay);
        assert_eq!(name, three);
        assert!(refused.expect_err("relay").starts_with("REFUSED"));
    }
}
