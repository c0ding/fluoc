use crate::logger::{ErrorLevel, ErrorValue, Logger};
use crate::paths;

use std::env::current_exe;
use std::path;
use std::process;

#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq)]
/// Position helper struct
///
/// Used to keep track of all tokens and nodes.
/// Note that there are no line numbers, a `\n` character counts as on character.
pub struct Pos {
    /// Start position in characters
    pub s: usize,

    /// End position in characters
    pub e: usize,

    /// Filename of pos
    /// The file comes from looking it up on the sourcemap
    pub filename_id: usize,
}

impl Pos {
    pub fn new(s: usize, e: usize, filename_id: usize) -> Self {
        Self { s, e, filename_id }
    }

    pub fn to_tuple(&self) -> (usize, usize) {
        (self.s, self.e)
    }

    pub fn calc(lower: Self, higher: Self) -> Self {
        Self {
            s: lower.s,
            e: higher.e,
            filename_id: lower.filename_id,
        }
    }
}

pub fn get_high_priority<'a>(errors: Vec<(ErrorValue, ErrorLevel)>) -> Vec<ErrorValue> {
    // The different errors to have different priorities
    // We want to show the errors with the highest priority
    // Show all of the errors that have the the same priority
    let max_error_priority = errors.iter().max_by_key(|x| x.1 as usize).unwrap().1 as usize;
    errors
        .into_iter()
        .filter(|error| error.1 as usize == max_error_priority)
        .map(|error| error.0)
        .collect()
}

pub fn error_or_other<T>(value: Result<T, Vec<ErrorValue>>, logger: Logger) -> T {
    match value {
        Ok(val) => val,
        Err(errors) => {
            for error in errors {
                logger.as_ref().borrow_mut().error(error);
            }
            logger.as_ref().borrow_mut().raise();
            process::exit(1);
        }
    }
}

pub fn display_duration(duration: std::time::Duration) -> String {
    let time_nano = duration.as_nanos();
    if time_nano < 1000 {
        format!("{:.2}ns", time_nano)
    } else if time_nano < 1000000 {
        format!("{:.2}µs", time_nano as f64 / 1000f64)
    } else if time_nano < 1e+9 as u128 {
        format!("{:.2}ms", time_nano as f64 / 1000000f64)
    } else {
        format!("{:.2}s", time_nano as f64 / 1e+9)
    }
}

lazy_static! {
    pub static ref CORE_LOC: path::PathBuf = {
        let mut core_path = path::PathBuf::from(file!());

        core_path.pop();

        core_path.push("fluo_core");
        core_path.push("core");
        core_path.push("core.fl");

        core_path
    };
    pub static ref STD_LOC: path::PathBuf = {
        let mut core_path = path::PathBuf::from(file!());

        core_path.pop();

        core_path.push("fluo_std");
        core_path.push("std");
        core_path.push("std.fl");

        core_path
    };
}

pub fn plural(n: usize, val: &str) -> String {
    if n == 1{
        format!("{}", val)
    } else {
        format!("{}s", val)
    }
}

