//! The stages of the message path a node can serve, and the one parse of a
//! node's declared feature capability (ADR-0056).
//!
//! A node declares which stages it serves — `receive`, `process`, `send`,
//! one or more — and nothing is inferred from what it is called. The words
//! become stages here and nowhere else, so an operator's door, a node's own
//! flag and a published record cannot disagree on what a word means (open
//! problem 25, row i: `node` parses, `cluster` places).
//!
//! The case rule is the owner's, 2026-09-24: the words are exact lowercase
//! only, so `RECEIVE` or `Send` is an unknown word. An unknown word is
//! REFUSED, naming the word and the words there are (ADR-0055); it is never
//! dropped.
//!
//! No other language writes the words or the parse again: the runtime's
//! library forwards [`Stage::WORDS`] and [`Stage::declared`] to the surfaces
//! as `xmip_stage_words_v1` and `xmip_stage_declared_v1` (`xmip_operate.h`
//! section 7), and `Xmip.Surface` and the estate's PowerShell module call
//! those (ADR-0056, amendment 2026-09-24, corrected the same day).

/// A stage of the message path, in path order.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum Stage {
    Receive,
    Process,
    Send,
}

impl Stage {
    /// Every stage, in message-path order.
    pub const ALL: [Stage; 3] = [Stage::Receive, Stage::Process, Stage::Send];

    /// The words a node may declare, in message-path order: each stage's
    /// [`name`](Self::name).
    pub const WORDS: [&'static str; 3] = ["receive", "process", "send"];

    /// The stage's canonical word, as it appears in a scope, a flag and a
    /// published record.
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Stage::Receive => "receive",
            Stage::Process => "process",
            Stage::Send => "send",
        }
    }

    /// The stage a word names, exactly and in lowercase, or `None` when no
    /// stage is called that.
    #[must_use]
    pub fn named(word: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|stage| stage.name() == word)
    }

    /// The stage a handoff goes on to: receive to process, process to send.
    /// Send closes the path and hands nothing on.
    #[must_use]
    pub const fn next(self) -> Option<Self> {
        match self {
            Self::Receive => Some(Self::Process),
            Self::Process => Some(Self::Send),
            Self::Send => None,
        }
    }

    /// Whether an operator may pause the stage: a Receive or a Send Location
    /// can be held; an Xmip Process runs off a subscription, and an operator
    /// pauses the Location that feeds it, not the Process itself.
    #[must_use]
    pub const fn pausable(self) -> bool {
        matches!(self, Self::Receive | Self::Send)
    }

    /// What a thing configured at the stage is called: a receive location,
    /// an Xmip Process, a send location (ADR-0027 clause 4).
    #[must_use]
    pub const fn location(self) -> &'static str {
        match self {
            Self::Receive => "receive location",
            Self::Process => "xmip process",
            Self::Send => "send location",
        }
    }

    /// The stages a declaration names: words separated by commas or by `+`,
    /// blanks ignored, returned in message-path order and each at most once.
    /// The empty declaration declares no stage, which is a real answer.
    ///
    /// # Errors
    ///
    /// When any word is no stage: REFUSED, naming every such word and the
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
                "REFUSED: no capability is called {}; a node declares {}, or nothing at all.",
                strangers.join(", "),
                Self::WORDS.join(", ")
            ));
        }
        Ok(Self::ALL
            .into_iter()
            .filter(|stage| words.contains(&stage.name()))
            .collect())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_words_are_the_stages_names_in_path_order() {
        assert_eq!(Stage::ALL.map(Stage::name), Stage::WORDS);
        assert_eq!(Stage::Receive.next(), Some(Stage::Process));
        assert_eq!(Stage::Send.next(), None);
    }

    #[test]
    fn a_location_can_be_paused_and_a_process_cannot() {
        assert_eq!(Stage::ALL.map(Stage::pausable), [true, false, true]);
        assert_eq!(
            Stage::ALL.map(Stage::location),
            ["receive location", "xmip process", "send location"]
        );
    }

    #[test]
    fn a_declaration_reads_in_path_order_once_each_and_empty_is_none() {
        assert_eq!(
            Stage::declared(" send + receive ,, receive"),
            Ok(vec![Stage::Receive, Stage::Send])
        );
        assert_eq!(Stage::declared(""), Ok(Vec::new()));
        assert_eq!(Stage::declared(" , + "), Ok(Vec::new()));
    }

    #[test]
    fn a_word_is_exact_lowercase_and_any_other_case_is_refused() {
        assert_eq!(
            Stage::declared("send + receive"),
            Ok(vec![Stage::Receive, Stage::Send])
        );
        assert_eq!(
            Stage::declared("Send + RECEIVE"),
            Err(
                "REFUSED: no capability is called Send, RECEIVE; a node declares receive, \
                 process, send, or nothing at all."
                    .to_string()
            )
        );
        assert!(Stage::named("RECEIVE").is_none());
        assert_eq!(Stage::named("receive"), Some(Stage::Receive));
    }

    #[test]
    fn an_unknown_word_is_refused_by_name_never_dropped() {
        let refusal = Stage::declared("receive,relay+hold").expect_err("relay is no stage");
        assert_eq!(
            refusal,
            "REFUSED: no capability is called relay, hold; a node declares receive, \
             process, send, or nothing at all."
        );
        assert!(Stage::named("relay").is_none());
        assert!(Stage::named("").is_none());
    }
}
