use std::fmt;

#[derive(Debug)]
pub enum NwwmError {
    DisplayUnavailable,
    InitError,
    ScreenGrabError,
    XCBConnError,
    MapError,
}

impl fmt::Display for NwwmError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            NwwmError::DisplayUnavailable => {
                write!(
                    f,
                    "[nwwm] error: failed to connect to X. check your DISPLAY environment variable (is X11 running?)"
                )
            }
            NwwmError::InitError => {
                write!(f, "[nwwm] error: another window manager is already running")
            }
            NwwmError::ScreenGrabError => {
                write!(f, "[nwwm] error: failed to get your screen")
            }
            NwwmError::XCBConnError => {
                write!(f, "[nwwm] error: XCB connection error")
            }
            NwwmError::MapError => {
                write!(f, "[nwwm] error: error encountered while mapping window")
            }
        }
    }
}

impl std::error::Error for NwwmError {}
