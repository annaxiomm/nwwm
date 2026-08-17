use xkbcommon::xkb;

use crate::{err::NwwmError, logger::LogLevel, tile::Layout, wm::WindowManager};
use std::process::{Command, Stdio};

#[derive(Clone)]
pub enum Action {
    FocusNext,
    CloseWindow,
    SetLayout(Layout),
    Exec(String),
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

        self.keysym == keysym.raw() && self.modifiers == state
    }
}

impl WindowManager {
    pub fn grab_keys(&self) {
        self.logger.log("grabbing keys...", LogLevel::Debug);
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
                let cookie = self.conn.send_request_checked(&xcb::x::GrabKey {
                    owner_events: false,
                    grab_window: self.ewmh.root,
                    modifiers: keybind.modifiers,
                    key: keycode.raw() as u8,
                    pointer_mode: xcb::x::GrabMode::Async,
                    keyboard_mode: xcb::x::GrabMode::Async,
                });

                match self.conn.check_request(cookie) {
                    Ok(_) => {}
                    Err(e) => eprintln!("[nwwm] error: {:?}", e),
                }
            }
        }

        self.conn.flush().unwrap();
    }

    pub fn run_action(&mut self, action: Action) -> Result<(), NwwmError> {
        match action {
            Action::FocusNext => self.focus_next()?,
            Action::SetLayout(layout) => self.set_layout(layout)?,
            Action::CloseWindow => {
                if let Some(win) = self.focused {
                    self.close_window(win)?;
                }
            }
            Action::Exec(command) => {
                let command_cloned = command.clone();
                if let Err(_) = self.exec_command(command) {
                    self.logger.log(
                        format!(
                            "failed to spawn command \"{}\"",
                            command_cloned.split(" ").next().unwrap()
                        )
                        .as_str(),
                        LogLevel::Warn,
                    );
                }
            }
        };
        Ok(())
    }

    pub fn exec_command(&self, command: String) -> Result<(), NwwmError> {
        let mut parts = command.split_whitespace();
        let Some(program) = parts.next() else {
            return Ok(());
        };

        Command::new(program)
            .args(parts)
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .map_err(|_| NwwmError::SpawnCommandError)?;
        Ok(())
    }
}
