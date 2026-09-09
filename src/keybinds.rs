use xkbcommon::xkb;

use crate::{err::NwwmError, logger::LogLevel, tile::Layout, wm::WindowManager};
use std::process::{Command, Stdio};

#[derive(Clone)]
pub enum Action {
    FocusNext,
    CloseWindow,
    SetLayout(Layout),
    Exec(String),
    SwitchWorkspace(usize),

    Quit,
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
            Action::SwitchWorkspace(id) => self.switch_workspace(id)?,
            Action::CloseWindow => {
                if let Some(win) = self.focused {
                    self.close_window(win)?;
                }
            }
            Action::Exec(command) => {
                let command_cloned = command.clone();
                if self.exec_command(command).is_err() {
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
            Action::Quit => {
                self.quit();
            }
        };
        Ok(())
    }

    pub fn switch_workspace(&mut self, workspace_id: usize) -> Result<(), NwwmError> {
        if workspace_id > self.num_workspaces || workspace_id == 0 {
            self.logger.log(
                format!("workspace index \"{}\" out of bounds", workspace_id).as_str(),
                LogLevel::Error,
            );
            return Ok(());
        }

        if workspace_id == self.current_workspace + 1 {
            return Ok(());
        }

        self.unmap_workspace(self.current_workspace);
        self.current_workspace = workspace_id - 1;
        self.map_workspace(self.current_workspace);

        if !self.workspaces[self.current_workspace].windows.is_empty() {
            self.focus_window(self.workspaces[self.current_workspace].windows[0].id)?;
        } else {
            self.unfocus();
        }

        self.ewmh
            .update_current_desktop(&self.conn, self.current_workspace as u32);

        self.tile()?;

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

    fn unmap_workspace(&self, id: usize) {
        let workspace = &self.workspaces[id];
        workspace
            .windows
            .iter()
            .for_each(|w| self.unmap_window(w.id));
    }

    fn map_workspace(&self, id: usize) {
        let workspace = &self.workspaces[id];
        workspace.windows.iter().for_each(|w| self.map_window(w.id));
    }

    fn unmap_window(&self, window: xcb::x::Window) {
        let cookie = self
            .conn
            .send_request_checked(&xcb::x::UnmapWindow { window });

        if let Err(e) = self.conn.check_request(cookie) {
            println!("UnmapWindow failed: {e}");
        }
    }
}
