static mut IS_VERBOSE: bool = false;

use chrono::prelude::*;

use std::thread;

pub fn print_datetime() {
    print!(
        "{}",
        Local::now().format("%Y-%m-%d %H:%M:%S%.6f").to_string()
    );
}

pub fn print_thread_id() {
    print!(" ({:?}): ", thread::current().id());
}

pub fn set_verbose(v: bool) {
    unsafe {
        IS_VERBOSE = v;
    }
}

pub fn is_verbose() -> bool {
    unsafe { IS_VERBOSE }
}

#[macro_export]
macro_rules! vprintln {
    () => (if crate::print::is_verbose() { std::print!("\n"); });
    ($($arg:tt)*) => {
        if crate::print::is_verbose() {
            crate::print::print_datetime();
            crate::print::print_thread_id();
            println!($($arg)*);
        }
    };
}

#[macro_export]
macro_rules! veprintln {
    () => (if crate::print::is_verbose() { std::print!("\n"); });
    ($($arg:tt)*) => {
        if crate::print::is_verbose() {
            crate::print::print_datetime();
            crate::print::print_thread_id();
            eprintln!($($arg)*);
        }
    };
}
