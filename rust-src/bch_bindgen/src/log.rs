use std::sync::atomic::{AtomicU8, Ordering};

pub const MUTE: u8 = 0;
pub const ERROR: u8 = 1;
pub const INFO: u8 = 2;
pub const DEBUG: u8 = 3;

// error level by default
pub static VERBOSE: AtomicU8 = AtomicU8::new(ERROR);

#[inline]
pub fn set_verbose_level(level: u8) {
    VERBOSE.store(level, Ordering::SeqCst);
}

pub fn max_level() -> u8 {
    VERBOSE.load(Ordering::SeqCst)
}

#[macro_export]
macro_rules! info {
    ($($arg:tt)*) => {
        if 2 <= $crate::log::max_level() {
            println!("{} {} {}",
                " INFO".green(),
                format!("{}:", module_path!()).bright_black(),
                format_args!($($arg)*)
            );
        }
    }
}

#[macro_export]
macro_rules! debug {
    ($($arg:tt)*) => {
        if 3 <= $crate::log::max_level() {
            println!("{} {} {}",
                "DEBUG".bright_blue(),
                format!("{}:", module_path!()).bright_black(),
                format_args!($($arg)*)
            );
        }
    }
}

#[macro_export]
macro_rules! error {
    ($($arg:tt)*) => {
        if 1 <= $crate::log::max_level() {
            println!("{} {} {}",
                "ERROR".bright_red(),
                format!("{}:", module_path!()).bright_black(),
                format_args!($($arg)*)
            );
        }
    }
}
