//! A declaration read back: what a System Process said of itself, which
//! process said it, when, from where on disk, and the file it stands in
//! (ADR-0053 clause 3). The one reader of the file
//! [`Declaration::declare_in`] writes; the runtime's library forwards
//! [`standing`] to the surfaces as `xmip_process_declarations_v1`.
//!
//! Whether the process that wrote a file still runs is not decided here: a
//! reader that can see the operating system's processes drops the files of
//! the ones that are gone.

use std::fs;
use std::path::{Path, PathBuf};

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

        let mut declaration = Declaration::new(name?, location?, purpose?);
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

/// Every declaration that stands in `directory`, one per `xmip-*.toml` file
/// that holds one, by file name. A directory that is not there holds none;
/// a file that is not a declaration, or went while it was read — a process
/// ending takes its file with it — is passed over.
#[must_use]
pub fn standing(directory: &Path) -> Vec<Standing> {
    let Ok(entries) = fs::read_dir(directory) else {
        return Vec::new();
    };

    let mut files: Vec<PathBuf> = entries
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .filter(|path| {
            path.file_name()
                .and_then(|name| name.to_str())
                .is_some_and(|name| {
                    name.starts_with("xmip-")
                        && Path::new(name)
                            .extension()
                            .is_some_and(|extension| extension.eq_ignore_ascii_case("toml"))
                })
        })
        .collect();
    files.sort();

    files
        .into_iter()
        .filter_map(|file| {
            let text = fs::read_to_string(&file).ok()?;
            Standing::read(&text, file)
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;

    fn scratch(name: &str) -> PathBuf {
        let directory = env::temp_dir().join("xmip-standing-test").join(name);
        let _ = fs::remove_dir_all(&directory);
        directory
    }

    #[test]
    fn what_a_process_declared_is_read_back_whole() {
        let directory = scratch("whole");
        let declared = Declaration::new("xmip-playground-node", "a \"b\"\\c", Purpose::Test)
            .with("online", "true")
            .expect("a bare key")
            .declare_in(&directory)
            .expect("declared");

        let read = standing(&directory);

        assert_eq!(read.len(), 1);
        assert_eq!(read[0].declaration.location, "a \"b\"\\c");
        assert_eq!(read[0].declaration.purpose, Purpose::Test);
        assert_eq!(
            read[0].declaration.said,
            vec![("online".to_string(), "true".to_string())]
        );
        assert_eq!(read[0].pid, std::process::id());
        assert_eq!(read[0].file, declared.file());
    }

    #[test]
    fn a_file_that_is_no_declaration_and_a_missing_directory_hold_none() {
        let directory = scratch("strangers");
        fs::create_dir_all(&directory).expect("made");
        fs::write(directory.join("xmip-a-1.toml"), "name = \"x\"\n").expect("no pid");
        fs::write(directory.join("xmip-b-2.toml"), "not toml at all").expect("written");
        fs::write(
            directory.join("xmip-c-3.toml"),
            "name = \"x\"\nlocation = \"\"\npurpose = \"production\"\npid = 3\n",
        )
        .expect("an unknown purpose");
        fs::write(directory.join("other-4.toml"), "").expect("not xmip-*");

        assert!(standing(&directory).is_empty());
        assert!(standing(&directory.join("nowhere")).is_empty());
    }
}
