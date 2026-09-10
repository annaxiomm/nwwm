use crate::{err::NwwmError, logger::LogLevel};

mod atoms;
mod config;
mod err;
mod ewmh;
mod handlers;
mod keybinds;
mod logger;
mod tile;
mod wm;

fn main() {
    // janky ahh error handling
    if let Err(e) = nwwm() {
        eprintln!("{e}");
        std::process::exit(1);
    }
}

fn nwwm() -> Result<(), NwwmError> {
    let args: Vec<String> = std::env::args().collect();
    let mut verbose = false;
    if args.contains(&"-v".to_string()) || args.contains(&"--verbose".to_string()) {
        verbose = true;
    }

    let logger = logger::Logger::new(true, verbose);
    logger.log("starting nwwm...", LogLevel::Info);

    let mut wm = wm::WindowManager::new(logger)?;
    wm.run()?;

    Ok(())
}
