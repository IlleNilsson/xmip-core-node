//! What a System Process Xmip owns says of itself: its name, its location
//! and its purpose (ADR-0053 clause 3), and whatever else it was started
//! with that a reader would otherwise have to dig out of its command line.
//!
//! The name finds a process — every one is `xmip` or `xmip-<what>` — and the
//! declaration says what it is for. This is what a declaration is and the
//! text it is written as ([`Declaration::to_toml`]); the file a process
//! writes it to where it starts and takes away where it ends, its
//! directory, and the listing of every declaration standing there are the
//! runtime's (`xmip-core-runtime`'s `declared`), which forwards them to the
//! surfaces as `xmip_process_declare_v1` and `xmip_process_declarations_v1`
//! (`xmip_operate.h` section 13). A definition opens no file (ADR-0058
//! clause 1, amendment 2026-11-13). Until 2026-09-27 .NET wrote the file
//! again and PowerShell read it with a TOML reader of its own.

use codec::toml::quote;
use std::fmt::Write as _;

/// The keys every declaration writes, which nothing else it says may take.
pub const KEYS: [&str; 6] = ["name", "location", "purpose", "pid", "started_unix", "path"];

/// What a System Process is for.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Purpose {
    /// The Essentials and everything it spawns, and whatever a test started.
    Test,
    /// Everything else: the product doing its work.
    Runtime,
}

impl Purpose {
    /// The words a process may declare, exact and lowercase.
    pub const WORDS: [&'static str; 2] = ["test", "runtime"];

    /// The word written in a declaration.
    #[must_use]
    pub const fn word(self) -> &'static str {
        match self {
            Self::Test => "test",
            Self::Runtime => "runtime",
        }
    }

    /// The purpose a word declares, exactly and in lowercase, as a role's
    /// word is (`NodeRole::declared`).
    ///
    /// # Errors
    ///
    /// No purpose is called that: REFUSED, naming the word and the words
    /// there are (ADR-0055). A word is never taken for runtime by default.
    pub fn declared(word: &str) -> Result<Self, String> {
        match word {
            "test" => Ok(Self::Test),
            "runtime" => Ok(Self::Runtime),
            other => Err(format!(
                "REFUSED: no purpose is called {other:?}; a process declares {}.",
                Self::WORDS.join(" or ")
            )),
        }
    }
}

/// What a System Process says of itself.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Declaration {
    /// What it is: `xmip` or `xmip-<what>`, the name the operating system
    /// schedules.
    pub name: String,
    /// Where in Xmip it belongs: the scope it serves, or the surface it
    /// reads where it serves none.
    pub location: String,
    /// Test or runtime.
    pub purpose: Purpose,
    /// What else it says of itself, key to text, in the order it said it: a
    /// Essentials node its flags, so no reader parses its command line.
    pub said: Vec<(String, String)>,
}

/// Why `name` is no System Process's name, if it is not: `xmip`, or `xmip-`
/// and letters, digits and hyphens (ADR-0053 clauses 1 and 4; the cluster
/// and the node in an Essentials name keep their case). The name becomes
/// the declaration's file name, so nothing else ever reaches the path.
fn refusal(name: &str) -> Option<String> {
    let shaped = name == "xmip"
        || name.strip_prefix("xmip-").is_some_and(|what| {
            !what.is_empty()
                && what
                    .chars()
                    .all(|letter| letter.is_ascii_alphanumeric() || letter == '-')
        });
    (!shaped).then(|| {
        format!(
            "REFUSED: a System Process is named xmip or xmip-<what>, letters, digits and \
             hyphens: not {name:?}"
        )
    })
}

impl Declaration {
    /// The declaration of the process `name`, at `location`, for `purpose`.
    ///
    /// # Errors
    ///
    /// `name` is not `xmip` or `xmip-<what>`, letters, digits and hyphens:
    /// REFUSED, in words, since the name is the declaration's file name.
    pub fn new(
        name: impl Into<String>,
        location: impl Into<String>,
        purpose: Purpose,
    ) -> Result<Self, String> {
        let name = name.into();
        if let Some(refused) = refusal(&name) {
            return Err(refused);
        }
        Ok(Self {
            name,
            location: location.into(),
            purpose,
            said: Vec::new(),
        })
    }

    /// The declaration saying one thing more.
    ///
    /// # Errors
    ///
    /// The key is not a bare TOML key — letters, digits, `_` and `-` — or
    /// is one of [`KEYS`], which every declaration writes itself.
    pub fn with(mut self, key: &str, value: impl Into<String>) -> Result<Self, String> {
        let bare = !key.is_empty()
            && key
                .chars()
                .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-');

        if !bare || KEYS.contains(&key) {
            return Err(format!(
                "REFUSED: a declaration cannot say {key:?}; a key is a bare word and none of {}.",
                KEYS.join(", ")
            ));
        }

        self.said.push((key.to_string(), value.into()));
        Ok(self)
    }

    /// The declaration as the TOML a reader takes: the three things, which
    /// process said so, when, and from where on disk, then what else it said.
    #[must_use]
    pub fn to_toml(&self, pid: u32, started_unix: u64, path: &str) -> String {
        let mut text = format!(
            "name = {}\nlocation = {}\npurpose = {}\npid = {pid}\n\
             started_unix = {started_unix}\npath = {}\n",
            quote(&self.name),
            quote(&self.location),
            quote(self.purpose.word()),
            quote(path),
        );

        for (key, value) in &self.said {
            // Writing into a String cannot fail.
            let _ = writeln!(text, "{key} = {}", quote(value));
        }

        text
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_declaration_says_its_three_things_and_which_process_said_them() {
        // The name is whatever the operator called the node, and nothing here
        // reads anything out of it (ADR-0053).
        let cluster = configure::fixture::test_cluster();
        let name = format!(
            "xmip-essentials-{}-node-{}",
            cluster.name,
            cluster.node(0).name
        );
        let location = cluster.node_scope(0);
        let declaration = Declaration::new(&name, &location, Purpose::Test).expect("a name");

        let text = declaration.to_toml(4242, 1_800_000_000, "node");

        assert!(text.contains(&format!("name = \"{name}\"")), "{text}");
        assert!(
            text.contains(&format!("location = \"{location}\"")),
            "{text}"
        );
        assert!(text.contains("purpose = \"test\""), "{text}");
        assert!(text.contains("pid = 4242"), "{text}");
        assert!(text.contains("started_unix = 1800000000"), "{text}");
    }

    #[test]
    fn a_windows_path_and_a_quote_survive_as_toml() {
        let declaration =
            Declaration::new("xmip", "a \"quoted\" place", Purpose::Runtime).expect("a name");
        let text = declaration.to_toml(7, 1_800_000_000, "C:\\Program Files\\Xmip\\xmip.exe");

        assert!(
            text.contains("location = \"a \\\"quoted\\\" place\""),
            "{text}"
        );
        assert!(
            text.contains("path = \"C:\\\\Program Files\\\\Xmip\\\\xmip.exe\""),
            "{text}"
        );
        assert!(text.contains("purpose = \"runtime\""), "{text}");
    }

    #[test]
    fn every_control_character_is_escaped_as_toml_requires() {
        let declaration =
            Declaration::new("xmip", "a\rb\tc\u{0}d\u{7f}", Purpose::Runtime).expect("a name");
        let text = declaration.to_toml(7, 1_800_000_000, "C:\\x\ny");

        assert!(
            text.contains(r#"location = "a\rb\tc\u0000d\u007F""#),
            "{text}"
        );
        assert!(text.contains(r#"path = "C:\\x\ny""#), "{text}");
        assert_eq!(text.lines().count(), 6, "one line per key: {text}");
    }

    #[test]
    fn what_else_a_process_says_follows_the_six_and_takes_none_of_their_keys() {
        let scope = configure::fixture::test_cluster().scope();
        let declaration = Declaration::new("xmip-essentials-node", &scope, Purpose::Test)
            .and_then(|said| said.with("stress", "calm"))
            .and_then(|said| said.with("rounds", "0"))
            .expect("two bare keys");
        let text = declaration.to_toml(7, 1, "node");

        assert!(
            text.ends_with("stress = \"calm\"\nrounds = \"0\"\n"),
            "{text}"
        );

        for refused in ["pid", "name", "a b", "", "x=y"] {
            let said = Declaration::new("xmip", "", Purpose::Runtime)
                .and_then(|declaration| declaration.with(refused, "1"));
            assert!(
                said.is_err_and(|why| why.starts_with("REFUSED")),
                "{refused}"
            );
        }
    }

    #[test]
    fn a_name_that_would_leave_the_directory_is_refused() {
        for refused in [
            "../x",
            "xmip-../x",
            "xmip-a/b",
            "xmip-a\\b",
            "xmip-.",
            "xmip-",
            "xmipx",
            "",
            "/etc/x",
        ] {
            let why = Declaration::new(refused, "xmip:///", Purpose::Test).expect_err(refused);
            assert!(why.starts_with("REFUSED"), "{why}");
        }
        for named in ["xmip", "xmip-cli", "xmip-essentials-W1-roll"] {
            assert!(
                Declaration::new(named, "", Purpose::Test).is_ok(),
                "{named}"
            );
        }
    }

    #[test]
    fn a_purpose_is_one_of_two_exact_words_and_anything_else_is_refused() {
        assert_eq!(Purpose::declared("test"), Ok(Purpose::Test));
        assert_eq!(Purpose::declared("runtime"), Ok(Purpose::Runtime));

        for stranger in ["TEST", " test", "", "production"] {
            let refused = Purpose::declared(stranger).expect_err(stranger);
            assert!(refused.starts_with("REFUSED"), "{refused}");
            assert!(refused.contains("test or runtime"), "{refused}");
        }
    }
}
