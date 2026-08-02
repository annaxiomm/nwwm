use std::fmt;

#[derive(Debug)]
pub enum NwwmError {
    DisplayUnavailable,
    InitError,
    XCBConnError,
}

impl fmt::Display for NwwmError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            NwwmError::DisplayUnavailable => {
                write!(
                    f,
                    "[nwwm] failed to connect to X. check your DISPLAY environment variable (is X11 running?)"
                )
            }
            NwwmError::InitError => {
                write!(
                    f,
                    "[nwwm] failed to initialise nwwm. is another wm running?"
                )
            }
            NwwmError::XCBConnError => {
                write!(f, "[nwwm] XCB connection error")
            }
        }
    }
}

impl std::error::Error for NwwmError {}
