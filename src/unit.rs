//! A unit as the crate implementing it declares it (ADR-0066, amendment
//! 2026-10-22): its [`Declaration`] — its name, what it can be called as,
//! the stages of the message path it runs at, and its settings in
//! `xmip-core`'s one shape — and, where its code is linked, the [`Unit`]
//! that opens it for a step. The declaration is data and is written once
//! beside the code, so the manifest that lists it, the check of a step's
//! settings and the opening read the same one; a crate gives it without its
//! code, so `configure` knows the standard units without linking them.
//!
//! `T` is the capability's own trait: a prepare unit opens a
//! `Box<dyn Prepare>`, a transform unit a `Box<dyn Transform>`. A unit is
//! handed the stage it is opened at, and does there what that stage asks
//! (the owner, 2026-10-22: *It is just an enum away what it does in what
//! situation*).

use xcore::settings::{Applies, Given, Read, Settings};

use crate::Stage;

/// What a unit is, as data: what a manifest lists of it.
#[derive(Debug, PartialEq, Eq)]
pub struct Declaration {
    /// Its name, as a step entry's `unit` names it.
    pub name: &'static str,
    /// What it can be called as, by the manifest's word: `prepare`,
    /// `validate`, `promote`, `demote`, `transform`, `work-process`.
    pub capability: &'static str,
    /// The stages it runs at, in message-path order.
    pub stages: &'static [Stage],
    /// What a step gives it; `technology` is the unit's name.
    pub settings: &'static Settings,
}

impl Declaration {
    /// What is wrong with the declaration, one sentence each: its settings'
    /// problems, a technology that is not its name, no stage.
    #[must_use]
    pub fn problems(&self) -> Vec<String> {
        let mut problems: Vec<String> = self
            .settings
            .problems()
            .into_iter()
            .map(|problem| format!("{}: {problem}", self.name))
            .collect();
        if self.settings.technology != self.name {
            problems.push(format!(
                "{}: its settings name '{}'",
                self.name, self.settings.technology
            ));
        }
        if self.stages.is_empty() {
            problems.push(format!("{}: it runs at no stage", self.name));
        }
        problems
    }
}

/// How a unit is opened: at its stage, from what its step gave, read
/// through its declaration; or why not, in words.
pub type Open<T> = fn(Stage, &Read) -> Result<Box<T>, String>;

/// What opens a unit for a step: a [`Unit`] linked into the program, or an
/// adapter over a module loaded through the C ABI (ADR-0012), which opens
/// its unit through the module's table. What the runtime keeps and calls
/// is this, so a unit is opened alike however it came into the process.
pub trait Opener<T: ?Sized>: Send + Sync {
    /// The unit's name, as a step entry's `unit` names it.
    fn name(&self) -> &str;

    /// Open it for a step at `stage` that gave `given`.
    ///
    /// # Errors
    /// The unit does not run at `stage`, or refuses the settings, in words.
    fn open(&self, stage: Stage, given: &[(String, Given)]) -> Result<Box<T>, String>;
}

/// A unit linked into the program: its declaration and how it opens.
pub struct Unit<T: ?Sized + 'static> {
    /// What it is.
    pub declaration: &'static Declaration,
    open: Open<T>,
}

impl<T: ?Sized + 'static> Unit<T> {
    /// The unit `declaration` declares, opened by `open`.
    #[must_use]
    pub const fn new(declaration: &'static Declaration, open: Open<T>) -> Self {
        Self { declaration, open }
    }

    /// Open it for a step at `stage` that gave `given`.
    ///
    /// # Errors
    /// The unit does not run at `stage`; every setting its declaration
    /// refuses, each named; then what the unit refuses of the values.
    pub fn open(&self, stage: Stage, given: &[(String, Given)]) -> Result<Box<T>, String> {
        let declaration = self.declaration;
        if !declaration.stages.contains(&stage) {
            return Err(format!(
                "the unit '{}' does not run at {}",
                declaration.name,
                stage.name()
            ));
        }
        let read = declaration
            .settings
            .read(Applies::Both, given)
            .map_err(|refused| refused.to_string())?;
        (self.open)(stage, &read)
    }
}

impl<T: ?Sized + 'static> Opener<T> for Unit<T> {
    fn name(&self) -> &str {
        self.declaration.name
    }

    fn open(&self, stage: Stage, given: &[(String, Given)]) -> Result<Box<T>, String> {
        Unit::open(self, stage, given)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use xcore::settings::{Fixed, Kind, Presence, Setting};

    trait Multiplying {
        fn of(&self, value: i64) -> i64;
    }

    struct Times(i64);

    impl Multiplying for Times {
        fn of(&self, value: i64) -> i64 {
            value * self.0
        }
    }

    const TIMES: &Declaration = &Declaration {
        name: "Times",
        capability: "transform",
        stages: &[Stage::Receive, Stage::Send],
        settings: &Settings {
            technology: "Times",
            settings: &[Setting {
                name: "factor",
                kind: Kind::Integer {
                    minimum: 1,
                    maximum: 10,
                },
                presence: Presence::Default(Fixed::Integer(2)),
                meaning: "What the value is multiplied by.",
                applies: Applies::Both,
            }],
        },
    };

    fn open(stage: Stage, read: &Read) -> Result<Box<dyn Multiplying>, String> {
        let factor = read.integer("factor");
        Ok(Box::new(Times(if stage == Stage::Send {
            -factor
        } else {
            factor
        })))
    }

    const UNIT: Unit<dyn Multiplying> = Unit::new(TIMES, open);

    #[test]
    fn a_unit_opens_from_its_declaration_at_its_stage_and_says_where_it_does_not_run() {
        let opened = UNIT
            .open(Stage::Receive, &[("factor".into(), Given::Integer(3))])
            .expect("opens");
        assert_eq!(opened.of(2), 6);
        assert_eq!(UNIT.open(Stage::Send, &[]).expect("opens").of(2), -4);
        let refused = UNIT.open(Stage::Process, &[]).err().expect("refused");
        assert!(refused.contains("does not run at process"), "{refused}");
        let refused = UNIT
            .open(Stage::Receive, &[("speed".into(), Given::Integer(1))])
            .err()
            .expect("refused");
        assert!(refused.contains("speed"), "{refused}");
        assert_eq!(Opener::name(&UNIT), "Times");
    }

    #[test]
    fn a_declaration_says_what_is_wrong_with_it() {
        assert!(TIMES.problems().is_empty(), "{:?}", TIMES.problems());
        let nowhere = Declaration {
            name: "Other",
            capability: "transform",
            stages: &[],
            settings: TIMES.settings,
        };
        let problems = nowhere.problems();
        assert!(
            problems
                .iter()
                .any(|p| p.contains("its settings name 'Times'")),
            "{problems:?}"
        );
        assert!(
            problems.iter().any(|p| p.contains("no stage")),
            "{problems:?}"
        );
    }
}
