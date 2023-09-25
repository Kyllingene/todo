use std::fmt::Display;

pub const RED: &str = "\x1b[38;5;1m";
pub const YELLOW: &str = "\x1b[38;5;3m";
pub const BLUE: &str = "\x1b[38;5;6m";

pub const BOLD: &str = "\x1b[1m";
pub const ITALIC: &str = "\x1b[3m";
pub const RESET: &str = "\x1b[0m";

pub trait CleanFail<T> {
    fn fail(self, msg: impl Display) -> T;
}

impl<T, E: Display> CleanFail<T> for Result<T, E> {
    fn fail(self, msg: impl Display) -> T {
        match self {
            Ok(t) => t,
            Err(e) => {
                log::err(format!("{msg}: {e}"));
                std::process::exit(1);
            }
        }
    }
}

impl<T> CleanFail<T> for Option<T> {
    fn fail(self, msg: impl Display) -> T {
        match self {
            Some(t) => t,
            None => {
                log::err(msg);
                std::process::exit(1);
            }
        }
    }
}

#[allow(unused)]
pub mod log {
    use super::*;

    pub fn info(msg: impl Display) {
        eprintln!("{BLUE}{msg}{RESET}");
    }

    pub fn warn(msg: impl Display) {
        eprintln!("{YELLOW}{ITALIC}{msg}{RESET}");
    }

    pub fn err(err: impl Display) {
        eprintln!("{RED}{BOLD}{err}{RESET}");
    }
}
