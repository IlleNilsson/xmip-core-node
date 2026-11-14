//! A declaration read back: what a System Process said of itself, which
//! process said it, when, from where on disk, and the file it stands in
//! (ADR-0053 clause 3). The one reading of the text
//! [`Declaration::to_toml`] writes; finding the files in their directory
//! and reading each is the runtime's (`xmip-core-runtime`'s
//! `declared::standing`), which forwards them to the surfaces as
//! `xmip_process_declarations_v1` (ADR-0058 clause 1, amendment
//! 2026-11-13).

use std::path::PathBuf;

use codec::toml::unquote;

use crate::declaration::{Declaration, Purpose};

/// A declaration that stands in a directory.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Standing {
    /// What the process said of itself.
    pub declaration: Declaration,
    /// The process that said it.
    pub pid: u32,
    /// When, in seconds since the Unix epoch; 0 where it did not say.
    pub started_unix: u64,
    /// Its executable on disk; empty where it did not say.
    pub path: String,
    /// The file the declaration stands in.
    pub file: PathBuf,
}

impl Standing {
    /// The declaration a file's text holds, as `file`. None where it is not
    /// one: a name, a location, a purpose word and a pid are what make it
    /// one, and a line that is neither a key with a string nor a key with a
    /// whole number makes it none.
    #[must_use]
    pub fn read(text: &str, file: PathBuf) -> Option<Self> {
        let mut name = None;
        let mut location = None;
        let mut purpose = None;
        let mut pid = None;
        let mut started_unix = 0;
        let mut path = String::new();
        let mut said = Vec::new();

        for line in text.lines().map(str::trim) {
            if line.is_empty() || line.starts_with('#') {
                continue;
            }

            let (key, value) = line.split_once('=')?;
            let (key, value) = (key.trim(), value.trim());
            let text = if value.starts_with('"') {
                unquote(value).ok()?
            } else {
                value.parse::<u64>().ok()?.to_string()
            };

            match key {
                "name" => name = Some(text),
                "location" => location = Some(text),
                "purpose" => purpose = Purpose::declared(&text).ok(),
                "pid" => pid = text.parse::<u32>().ok(),
                "started_unix" => started_unix = text.parse().ok()?,
                "path" => path = text,
                other => said.push((other.to_string(), text)),
            }
        }

        let mut declaration = Declaration::new(name?, location?, purpose?).ok()?;
        declaration.said = said;

        Some(Self {
            declaration,
            pid: pid?,
            started_unix,
            path,
            file,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn what_a_process_declared_is_read_back_whole() {
        let declaration = Declaration::new("xmip-essentials-node", "a \"b\"\\c", Purpose::Test)
            .and_then(|declaration| declaration.with("online", "true"))
            .expect("a bare key");
        let file = PathBuf::from("xmip-essentials-node-7.toml");

        let read = Standing::read(&declaration.to_toml(7, 1_800_000_000, "node"), file.clone())
            .expect("a declaration");

        assert_eq!(
            read,
            Standing {
                declaration,
                pid: 7,
                started_unix: 1_800_000_000,
                path: "node".to_string(),
                file,
            }
        );
    }

    #[test]
    fn a_text_that_is_no_declaration_reads_as_none() {
        for stranger in [
            "name = \"x\"\n",
            "not toml at all",
            "name = \"x\"\nlocation = \"\"\npurpose = \"production\"\npid = 3\n",
            "",
        ] {
            assert_eq!(
                Standing::read(stranger, PathBuf::from("xmip-x-3.toml")),
                None,
                "{stranger}"
            );
        }
    }
}
