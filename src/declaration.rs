//! What a System Process Xmip owns says of itself: its name, its location
//! and its purpose (ADR-0053 clause 3).
//!
//! The name finds a process — every one is `xmip-<what>` — and the
//! declaration says what it is for. A process writes it where it starts, to
//! one file named for it and its pid, and takes it away where it ends; a
//! process that is killed leaves its file behind, and whoever lists the
//! declarations drops the ones whose process is gone. The directory is the
//! node's to say, in `XMIP_PROCESS_DIRECTORY`; unset, it is `xmip/process`
//! under the system's temporary directory, the same for every process on the
//! machine, so that a reader and a writer who were told nothing still meet.

use codec::toml::quote;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};
use std::{env, fs, io, process};

/// The environment variable that names the directory declarations are
/// written to.
pub const DIRECTORY_VARIABLE: &str = "XMIP_PROCESS_DIRECTORY";

/// What a System Process is for.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Purpose {
    /// The Playground and everything it spawns, and whatever a test started.
    Test,
    /// Everything else: the product doing its work.
    Runtime,
}

impl Purpose {
    /// The word written in a declaration.
    #[must_use]
    pub const fn word(self) -> &'static str {
        match self {
            Self::Test => "test",
            Self::Runtime => "runtime",
        }
    }

    /// The purpose a word names; anything that is not `test` is runtime,
    /// because a process is runtime unless what started it says otherwise.
    #[must_use]
    pub fn named(word: &str) -> Self {
        if word.trim().eq_ignore_ascii_case("test") {
            Self::Test
        } else {
            Self::Runtime
        }
    }
}

/// The three things a System Process says of itself.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Declaration {
    /// What it is: `xmip-<what>`, the name the operating system schedules.
    pub name: String,
    /// Where in Xmip it belongs: the scope it serves, or the surface it
    /// reads where it serves none.
    pub location: String,
    /// Test or runtime.
    pub purpose: Purpose,
}

impl Declaration {
    #[must_use]
    pub fn new(name: impl Into<String>, location: impl Into<String>, purpose: Purpose) -> Self {
        Self {
            name: name.into(),
            location: location.into(),
            purpose,
        }
    }

    /// Declare this process in the directory the node names.
    ///
    /// # Errors
    ///
    /// The directory could not be made or the file could not be written. A
    /// process that cannot declare itself still runs; its caller says so.
    pub fn declare(&self) -> io::Result<Declared> {
        self.declare_in(&directory())
    }

    /// Declare this process in a given directory.
    ///
    /// # Errors
    ///
    /// The directory could not be made or the file could not be written.
    pub fn declare_in(&self, directory: &Path) -> io::Result<Declared> {
        fs::create_dir_all(directory)?;

        let pid = process::id();
        let file = directory.join(format!("{}-{pid}.toml", self.name));
        let started = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_or(0, |since| since.as_secs());
        let path = env::current_exe()
            .map(|path| path.display().to_string())
            .unwrap_or_default();

        fs::write(&file, self.to_toml(pid, started, &path))?;
        Ok(Declared { file })
    }

    /// The declaration as the TOML a reader takes: the three things, and
    /// beside them which process said so, when, and from where on disk.
    #[must_use]
    pub fn to_toml(&self, pid: u32, started_unix: u64, path: &str) -> String {
        format!(
            "name = {}\nlocation = {}\npurpose = {}\npid = {pid}\n\
             started_unix = {started_unix}\npath = {}\n",
            quote(&self.name),
            quote(&self.location),
            quote(self.purpose.word()),
            quote(path),
        )
    }
}

/// A declaration that stands while this is held and is taken away when it
/// is dropped.
#[derive(Debug)]
pub struct Declared {
    file: PathBuf,
}

impl Declared {
    /// The file the declaration was written to.
    #[must_use]
    pub fn file(&self) -> &Path {
        &self.file
    }
}

impl Drop for Declared {
    fn drop(&mut self) {
        // Gone already is as good as removed.
        let _ = fs::remove_file(&self.file);
    }
}

/// Where declarations are written: what the node names, else `xmip/process`
/// under the system's temporary directory.
#[must_use]
pub fn directory() -> PathBuf {
    env::var_os(DIRECTORY_VARIABLE)
        .filter(|named| !named.is_empty())
        .map_or_else(
            || env::temp_dir().join("xmip").join("process"),
            PathBuf::from,
        )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn scratch(name: &str) -> PathBuf {
        let directory = env::temp_dir().join("xmip-declaration-test").join(name);
        let _ = fs::remove_dir_all(&directory);
        directory
    }

    #[test]
    fn a_declaration_stands_while_held_and_is_gone_when_dropped() {
        let directory = scratch("held");
        // The name is whatever the operator called the node, and nothing here
        // reads anything out of it (ADR-0053).
        let name = "xmip-playground-orders-node-edge-01";
        let declaration = Declaration::new(name, "xmip:///orders/node/edge-01", Purpose::Test);

        let declared = declaration.declare_in(&directory).expect("declared");
        let file = declared.file().to_path_buf();
        let text = fs::read_to_string(&file).expect("the file");

        assert!(
            file.ends_with(format!("{name}-{}.toml", process::id())),
            "{}",
            file.display()
        );
        assert!(text.contains(&format!("name = \"{name}\"")), "{text}");
        assert!(
            text.contains("location = \"xmip:///orders/node/edge-01\""),
            "{text}"
        );
        assert!(text.contains("purpose = \"test\""), "{text}");
        assert!(text.contains(&format!("pid = {}", process::id())), "{text}");

        drop(declared);
        assert!(!file.exists(), "taken away where the process ends");
    }

    #[test]
    fn a_windows_path_and_a_quote_survive_as_toml() {
        let declaration = Declaration::new("xmip-cli", "a \"quoted\" place", Purpose::Runtime);
        let text = declaration.to_toml(7, 1_800_000_000, "C:\\Program Files\\Xmip\\xmip-cli.exe");

        assert!(
            text.contains("location = \"a \\\"quoted\\\" place\""),
            "{text}"
        );
        assert!(
            text.contains("path = \"C:\\\\Program Files\\\\Xmip\\\\xmip-cli.exe\""),
            "{text}"
        );
        assert!(text.contains("purpose = \"runtime\""), "{text}");
    }

    #[test]
    fn every_control_character_is_escaped_as_toml_requires() {
        let declaration = Declaration::new("xmip-cli", "a\rb\tc\u{0}d\u{7f}", Purpose::Runtime);
        let text = declaration.to_toml(7, 1_800_000_000, "C:\\x\ny");

        assert!(
            text.contains(r#"location = "a\rb\tc\u0000d\u007F""#),
            "{text}"
        );
        assert!(text.contains(r#"path = "C:\\x\ny""#), "{text}");
        assert_eq!(text.lines().count(), 6, "one line per key: {text}");
    }

    #[test]
    fn a_process_is_runtime_unless_the_word_is_test() {
        assert_eq!(Purpose::named("test"), Purpose::Test);
        assert_eq!(Purpose::named(" TEST "), Purpose::Test);
        assert_eq!(Purpose::named("runtime"), Purpose::Runtime);
        assert_eq!(Purpose::named(""), Purpose::Runtime);
        assert_eq!(Purpose::named("production"), Purpose::Runtime);
    }
}
