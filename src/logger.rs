use std::sync::atomic::{AtomicBool, Ordering};

pub struct Logger {
    enabled: AtomicBool,
    verbose: AtomicBool,
}

#[allow(dead_code)]
pub enum LogLevel {
    Debug,
    Info,
    Warn,
    Error,
}

impl Logger {
    pub fn new(enabled: bool, verbose: bool) -> Self {
        Self {
            enabled: AtomicBool::new(enabled),
            verbose: AtomicBool::new(verbose),
        }
    }

    pub fn log(&self, text: &str, loglevel: LogLevel) {
        if self.enabled.load(Ordering::Relaxed) {
            match loglevel {
                LogLevel::Debug => {
                    if self.verbose.load(Ordering::Relaxed) {
                        println!("[nwwm] debug: {text}");
                    }
                }
                LogLevel::Info => {
                    println!("[nwwm] info: {text}")
                }
                LogLevel::Warn => {
                    println!("[nwwm] warning: {text}")
                }
                LogLevel::Error => {
                    eprintln!("[nwwm] error: {text}");
                    std::process::exit(1);
                }
            }
        }
    }
}
