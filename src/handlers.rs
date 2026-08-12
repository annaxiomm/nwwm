// HANDLERS.RS
// -----------
// event handlers go here to avoid clogging up wm.rs

use xkbcommon::xkb;

use crate::{
    err::NwwmError,
    wm::{Window, WindowManager, WindowState, WindowType},
};

impl WindowManager {
    pub fn on_map_request(&mut self, ev: xcb::x::MapRequestEvent) -> Result<(), NwwmError> {
        let window = ev.window();

        let cookie = self.conn.send_request(&xcb::x::GetProperty {
            delete: false,
            window: window,
            property: self.ewmh.atoms.net_wm_window_type,
            r#type: xcb::x::ATOM_ATOM,
            long_offset: 0,
            long_length: 32,
        });
        let reply = self
            .conn
            .wait_for_reply(cookie)
            .map_err(|_| NwwmError::MapError)?;
        let types: &[xcb::x::Atom] = reply.value();
        let window_type = self.get_type(types);
        let window_state = self.get_state(&window_type);

        if matches!(window_type, WindowType::Normal) {
            // don't manage dock or dialog windows
            self.workspaces[self.current_workspace] // Add to workspace before mapping so if MapWindow fails,
                .windows // we still know about it
                .push(Window {
                    id: window,
                    workspace: self.current_workspace,
                    window_type,
                    window_state,
                });
        }

        self.conn.send_request(&xcb::x::MapWindow { window });

        self.conn.send_request(&xcb::x::ConfigureWindow {
            window,
            value_list: &[xcb::x::ConfigWindow::BorderWidth(2)],
        });

        self.conn.send_request(&xcb::x::ChangeWindowAttributes {
            window,
            value_list: &[xcb::x::Cw::BorderPixel(self.config.border_unfocused)],
        });

        self.focus_window(window)?;
        self.tile()?;
        self.conn.flush().unwrap(); // without this, nothing happens

        Ok(())
    }

    pub fn on_destroy_notify(&mut self, ev: xcb::x::DestroyNotifyEvent) -> Result<(), NwwmError> {
        let window = ev.window();
        let windows = &self.workspaces[self.current_workspace].windows;
        let new_focus = if self.focused == Some(window) {
            windows
                .iter()
                .position(|w| w.id == window)
                .and_then(|position| {
                    position
                        .checked_sub(1)
                        .and_then(|prev| windows.get(prev))
                        .or_else(|| windows.get(position + 1))
                })
                .map(|w| w.id)
        } else {
            None
        };

        if Some(window) == self.focused {
            self.focused = None;
        }

        for workspace in &mut self.workspaces {
            workspace.windows.retain(|w| w.id != window);
        }

        if let Some(new_window) = new_focus {
            self.focus_window(new_window)?;
        }

        self.tile()?;

        Ok(())
    }

    pub fn on_button_press(&mut self, ev: xcb::x::ButtonPressEvent) -> Result<(), NwwmError> {
        self.focus_window(ev.child())?;
        self.conn.send_request(&xcb::x::AllowEvents {
            mode: xcb::x::Allow::ReplayPointer,
            time: ev.time(),
        });
        self.conn.flush().unwrap();

        Ok(())
    }

    pub fn on_key_press(&mut self, ev: xcb::x::KeyPressEvent) -> Result<(), NwwmError> {
        if let Some(keybind) = self
            .config
            .keybinds
            .iter()
            .find(|k| k.matches(&self.xkb_state, &ev))
        {
            self.run_action(keybind.action)?;
        }

        Ok(())
    }

    fn get_type(&self, types: &[xcb::x::Atom]) -> WindowType {
        if types.contains(&self.ewmh.atoms.net_wm_window_type_dialog) {
            return WindowType::Dialog;
        }

        if types.contains(&self.ewmh.atoms.net_wm_window_type_dock) {
            return WindowType::Dock;
        }

        if types.contains(&self.ewmh.atoms.net_wm_window_type_utility) {
            return WindowType::Utility;
        }

        println!("{:?}", types);

        WindowType::Normal
    }

    fn get_state(&self, window_type: &WindowType) -> WindowState {
        match window_type {
            WindowType::Dock | WindowType::Dialog | WindowType::Utility => WindowState::Floating,
            WindowType::Normal => WindowState::Tiled,
        }
    }
}
