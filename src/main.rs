use crate::err::NwwmError;

mod err;
mod handlers;
mod logger;
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

    let wm = wm::WindowManager::new(logger)?;
    wm.run()?;

    Ok(())
}
