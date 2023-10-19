#![allow(dead_code)]

use std::fmt::Display;

pub const RED: &str = "\x1b[38;5;1m";
pub const YELLOW: &str = "\x1b[38;5;3m";
pub const BLUE: &str = "\x1b[38;5;6m";

pub const BOLD: &str = "\x1b[1m";
pub const ITALIC: &str = "\x1b[3m";
pub const RESET: &str = "\x1b[0m";

pub fn info(msg: impl Display) {
    eprintln!("{BLUE}{msg}{RESET}");
}

pub fn warn(msg: impl Display) {
    eprintln!("{YELLOW}{ITALIC}{msg}{RESET}");
}

pub fn err(err: impl Display) {
    eprintln!("{RED}{BOLD}{err}{RESET}");
}

