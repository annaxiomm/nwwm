use xkbcommon::xkb;

use crate::{err::NwwmError, tile::Layout, wm::WindowManager};

#[derive(Copy, Clone)]
pub enum Action {
    FocusNext,
    SetLayout(Layout),
}

pub struct Keybind {
    pub modifiers: xcb::x::ModMask,
    pub keysym: u32,
    pub action: Action,
}

impl Keybind {
    pub fn matches(&self, xkb_state: &xkb::State, ev: &xcb::x::KeyPressEvent) -> bool {
        let state = xcb::x::ModMask::from_bits_truncate(ev.state().bits());

        let keycode = xkb::Keycode::new(ev.detail() as u32);
        let keysym = xkb_state.key_get_one_sym(keycode);

        println!(
            "keycode={}, keysym={}, expected={}, state={:?}, expected_state={:?}",
            ev.detail(),
            keysym.raw(),
            self.keysym,
            state,
            self.modifiers,
        );

        self.keysym == keysym.raw() && self.modifiers == state
    }
}

impl WindowManager {
    pub fn grab_keys(&self) {
        for keybind in &self.config.keybinds {
            let mut keycode = None;

            self.xkb_keymap.key_for_each(|_, kc| {
                for layout in 0..self.xkb_keymap.num_layouts_for_key(kc) {
                    for level in 0..self.xkb_keymap.num_levels_for_key(kc, layout) {
                        let syms = self.xkb_keymap.key_get_syms_by_level(kc, layout, level);

                        if syms.iter().any(|sym| sym.raw() == keybind.keysym) {
                            keycode = Some(kc);
                        }
                    }
                }
            });

            if let Some(keycode) = keycode {
                self.conn.send_request(&xcb::x::GrabKey {
                    owner_events: false,
                    grab_window: self.ewmh.root,
                    modifiers: keybind.modifiers,
                    key: keycode.raw() as u8,
                    pointer_mode: xcb::x::GrabMode::Async,
                    keyboard_mode: xcb::x::GrabMode::Async,
                });
            }
        }
    }

    pub fn run_action(&mut self, action: Action) -> Result<(), NwwmError> {
        match action {
            Action::FocusNext => self.focus_next()?,
            Action::SetLayout(layout) => self.set_layout(layout)?,
        };
        Ok(())
    }
}
