use std::fmt::Display;

pub const RED: &str = "\x1b[38;5;1m";
pub const YELLOW: &str = "\x1b[38;5;3m";
pub const BLUE: &str = "\x1b[38;5;6m";

pub const BOLD: &str = "\x1b[1m";
pub const ITALIC: &str = "\x1b[3m";
pub const RESET: &str = "\x1b[0m";

#[allow(unused)]
pub mod log {
    use super::*;

    pub fn info(msg: impl Display) {
        eprintln!("{BLUE}{msg}{RESET}");
    }

    pub fn warn(msg: impl Display) {
        eprintln!("{YELLOW}{BOLD}{msg}{RESET}");
    }

    pub fn err(msg: impl Display, err: impl Display) {
        eprintln!("{RED}{BOLD}{msg}:{RESET} {ITALIC}{RED}{err}{RESET}");
    }
}
