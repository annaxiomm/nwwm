use crate::err::NwwmError;

mod atoms;
mod config;
mod err;
mod ewmh;
mod handlers;
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
    println!("[nwwm] starting nwwm...");

    let logger = logger::Logger::new(true, true);

    let mut wm = wm::WindowManager::new(logger)?;
    wm.run()?;

    Ok(())
}
