use std::fs::File;
use std::io::Write;

const LOG_FILE: &'static str = "log.txt";
pub const LOG_ENABLE: bool = cfg!(debug_assertions);

#[macro_export]
macro_rules! log_error {
    ($($arg:tt)*) => {{
        if $crate::logger::log::LOG_ENABLE {
            let res: String = format!($($arg)*);
            $crate::logger::log::log("[ERRO]", &res[..]);
        }
    }}
}

#[macro_export]
macro_rules! log_warn {
    ($($arg:tt)*) => {{
        if $crate::logger::log::LOG_ENABLE {
            let res: String = format!($($arg)*);
            $crate::logger::log::log("[WARN]", &res[..]);
        }
    }}
}

#[macro_export]
macro_rules! log_info {
    ($($arg:tt)*) => {{
        if $crate::logger::log::LOG_ENABLE {
            let res: String = format!($($arg)*);
            $crate::logger::log::log("[INFO]", &res[..]);
        }
    }}
}

#[macro_export]
macro_rules! log_debug {
    ($($arg:tt)*) => {{
        if $crate::logger::log::LOG_ENABLE {
            let res: String = format!($($arg)*);
            $crate::logger::log::log("[DEBG]", &res[..]);
        }
    }}
}

pub fn log(header: &str, msg: &str) {
    println!("{} {}", header, msg);
    let file_handle_res = File::options().append(true).open(LOG_FILE).ok();
    if let Some(mut handle) = file_handle_res {
        writeln!(handle, "{} {}", header, msg).expect("Write to log failed");
    }
}

pub fn initialize() -> Result<File, std::io::Error> {
    return File::create(LOG_FILE)
}
