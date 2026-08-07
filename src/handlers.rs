// HANDLERS.RS
// -----------
// event handlers go here to avoid clogging up wm.rs

use xcb::x::Keycode;

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

        if !matches!(window_type, WindowType::Dock) {
            // don't manage dock windows
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

        self.conn.send_request(&xcb::x::GrabButton {
            owner_events: true,
            grab_window: window,
            event_mask: xcb::x::EventMask::BUTTON_PRESS,
            pointer_mode: xcb::x::GrabMode::Async,
            keyboard_mode: xcb::x::GrabMode::Async,
            confine_to: xcb::x::WINDOW_NONE,
            cursor: xcb::x::CURSOR_NONE,
            button: xcb::x::ButtonIndex::N1,
            modifiers: xcb::x::ModMask::ANY,
        });

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

        if self.focused == Some(window) {
            self.focused = None;
        }

        for workspace in &mut self.workspaces {
            workspace.windows.retain(|w| w.id != window);
        }

        self.tile()?;

        Ok(())
    }

    pub fn on_button_press(&mut self, ev: xcb::x::ButtonPressEvent) -> Result<(), NwwmError> {
        self.focus_window(ev.event())?;

        Ok(())
    }

    pub fn on_key_press(&mut self, ev: xcb::x::KeyPressEvent) -> Result<(), NwwmError> {
        if ev.detail() == Keycode::from(25) {
            self.focus_next()?;
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

        WindowType::Normal
    }

    fn get_state(&self, window_type: &WindowType) -> WindowState {
        match window_type {
            WindowType::Normal => WindowState::Tiled,
            WindowType::Dialog => WindowState::Floating,
            WindowType::Dock => WindowState::Floating,
        }
    }
}
