use crate::{err::NwwmError, wm::WindowManager};

#[derive(Copy, Clone)]
pub enum Action {
    FocusNext,
}

pub struct Keybind {
    pub modifiers: xcb::x::ModMask,
    pub keycode: u8,
    pub action: Action,
}

impl Keybind {
    pub fn matches(&self, ev: &xcb::x::KeyPressEvent) -> bool {
        let state = xcb::x::ModMask::from_bits_truncate(ev.state().bits());

        self.keycode == ev.detail() && self.modifiers == state
    }
}

impl WindowManager {
    pub fn grab_keys(&self) {
        for keybind in &self.config.keybinds {
            self.conn.send_request(&xcb::x::GrabKey {
                owner_events: false,
                grab_window: self.ewmh.root,
                modifiers: keybind.modifiers,
                key: keybind.keycode,
                pointer_mode: xcb::x::GrabMode::Async,
                keyboard_mode: xcb::x::GrabMode::Async,
            });
        }
    }

    pub fn run_action(&mut self, action: Action) -> Result<(), NwwmError> {
        match action {
            Action::FocusNext => self.focus_next()?,
        };
        Ok(())
    }
}
