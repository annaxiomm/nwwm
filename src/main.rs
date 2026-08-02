use crate::err::NwwmError;

mod err;
mod wm;

fn main() {
    // janky ahh error handling
    if let Err(e) = nwwm() {
        eprintln!("{e}");
        std::process::exit(1);
    }
}

fn nwwm() -> Result<(), NwwmError> {
    println!("[nwwm] hello!");

    let wm = wm::WindowManager::new()?;
    wm.run()?;

    Ok(())
}
