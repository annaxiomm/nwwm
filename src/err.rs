use std::fmt;

#[derive(Debug)]
pub enum NwwmError {
    DisplayUnavailable,
    InitError,
    ScreenGrabError,
    XCBConnError,
    MapError,
    XKBError,
    SpawnCommandError,
    HexFormatError,
    KeyBindError,
    ModError,
    ParseActionError,
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
            NwwmError::XKBError => {
                write!(f, "[nwwm] error: error encountered while initialising XKB")
            }
            NwwmError::SpawnCommandError => {
                write!(
                    f,
                    "[nwwm] error: could not spawn command (this shouldn't be fatal - open an issue on GitHub)"
                )
            }
            NwwmError::HexFormatError => {
                write!(
                    f,
                    "[nwwm] error: colours should be in #RRGGBB format (this shouldn't be fatal - open an issue on GitHub"
                )
            }
            NwwmError::KeyBindError => {
                write!(
                    f,
                    "[nwwm] error: invalid keybind (this shouldn't be fatal - open an issue on GitHub"
                )
            }
            NwwmError::ModError => {
                write!(
                    f,
                    "[nwwm] error: invalid modifier in keybind (this shouldn't be fatal - open an issue on GitHub"
                )
            }
            NwwmError::ParseActionError => {
                write!(
                    f,
                    "[nwwm] error: failed to parse action (this shouldn't be fatal - open an issue on GitHub"
                )
            }
        }
    }
}

impl std::error::Error for NwwmError {}
