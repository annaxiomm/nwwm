// HANDLERS.RS
// -----------
// event handlers go here to avoid clogging up wm.rs

use crate::{
    err::NwwmError,
    wm::{Dock, Strut, Window, WindowManager, WindowState, WindowType},
};

impl WindowManager {
    pub fn on_map_request(&mut self, ev: xcb::x::MapRequestEvent) -> Result<(), NwwmError> {
        let window = ev.window();
        let window_type = self.get_window_type(window)?;

        if matches!(window_type, WindowType::Dock) {
            self.handle_dock(window);
            self.map_window(window);

            self.recalculate_screen_area();
            self.tile()?;

            self.conn.flush().unwrap();
            return Ok(());
        }

        self.handle_window(window, window_type);
        self.map_window(window);
        self.focus_window(window)?;
        self.ewmh.update_client_list(&self.conn, &self.clients);

        self.tile()?;
        self.conn.flush().unwrap();
        Ok(())
    }

    pub fn on_destroy_notify(&mut self, ev: xcb::x::DestroyNotifyEvent) -> Result<(), NwwmError> {
        let window = ev.window();
        let windows = &self.workspaces[self.current_workspace].windows;

        // managed windows
        if windows.iter().any(|w| w.id == window) {
            self.destroy_managed_window(window)?;
            self.ewmh.update_client_list(&self.conn, &self.clients);

            self.tile()?;

            return Ok(());
        }

        // dock windows
        if self.docks.iter().any(|d| d.id == window) {
            self.docks.retain(|d| d.id != window);
            self.recalculate_screen_area();
            self.tile()?;
            return Ok(());
        }

        Ok(())
    }

    pub fn on_config_request(
        &mut self,
        ev: xcb::x::ConfigureRequestEvent,
    ) -> Result<(), NwwmError> {
        let window = ev.window();

        if !self
            .workspaces
            .iter()
            .any(|ws| ws.windows.iter().any(|w| w.id == window))
            && !self.docks.iter().any(|d| d.id == window)
        {
            return Ok(());
        }

        let window_type = self.get_window_type(window)?;
        let window_state = self.get_state(&window_type);

        if matches!(window_state, WindowState::Floating) {
            let mut values = Vec::new();
            if ev.value_mask().contains(xcb::x::ConfigWindowMask::X) {
                values.push(xcb::x::ConfigWindow::X(ev.x() as i32));
            }
            if ev.value_mask().contains(xcb::x::ConfigWindowMask::Y) {
                values.push(xcb::x::ConfigWindow::Y(ev.y() as i32));
            }
            if ev.value_mask().contains(xcb::x::ConfigWindowMask::WIDTH) {
                values.push(xcb::x::ConfigWindow::Width(ev.width() as u32));
            }
            if ev.value_mask().contains(xcb::x::ConfigWindowMask::HEIGHT) {
                values.push(xcb::x::ConfigWindow::Height(ev.height() as u32));
            }
            let cookie = self.conn.send_request_checked(&xcb::x::ConfigureWindow {
                window,
                value_list: &values,
            });

            if let Err(e) = self.conn.check_request(cookie) {
                println!("ConfigureWindow failed: {e:?}");
            }
        }

        self.conn.flush().unwrap();

        Ok(())
    }

    pub fn on_button_press(&mut self, ev: xcb::x::ButtonPressEvent) -> Result<(), NwwmError> {
        self.conn.send_request(&xcb::x::AllowEvents {
            mode: xcb::x::Allow::ReplayPointer,
            time: ev.time(),
        });
        if ev.child() != xcb::x::WINDOW_NONE {
            // if the window isn't managed don't try to focus it
            if let Some(window) = self.workspaces[self.current_workspace]
                .windows
                .iter()
                .find(|w| w.id == ev.child())
            {
                match window.window_type {
                    WindowType::Dock => {}
                    _ => self.focus_window(ev.child())?,
                }
            }
        }

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
            self.run_action(keybind.action.clone())?;
        }

        Ok(())
    }

    fn get_window_type(&self, window: xcb::x::Window) -> Result<WindowType, NwwmError> {
        let cookie = self.conn.send_request(&xcb::x::GetProperty {
            delete: false,
            window,
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

        if types.contains(&self.ewmh.atoms.net_wm_window_type_dialog) {
            return Ok(WindowType::Dialog);
        }
        if types.contains(&self.ewmh.atoms.net_wm_window_type_dock) {
            return Ok(WindowType::Dock);
        }
        if types.contains(&self.ewmh.atoms.net_wm_window_type_utility) {
            return Ok(WindowType::Utility);
        }

        Ok(WindowType::Normal)
    }

    fn get_state(&self, window_type: &WindowType) -> WindowState {
        match window_type {
            WindowType::Dock | WindowType::Dialog | WindowType::Utility => WindowState::Floating,
            WindowType::Normal => WindowState::Tiled,
        }
    }

    fn map_window(&self, window: xcb::x::Window) {
        self.conn.send_request(&xcb::x::MapWindow { window });
    }

    fn destroy_managed_window(&mut self, window: xcb::x::Window) -> Result<(), NwwmError> {
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
        self.clients.retain(|w| w.id != window);

        if let Some(new_window) = new_focus {
            self.focus_window(new_window)?;
        };

        Ok(())
    }

    fn handle_window(&mut self, window: xcb::x::Window, window_type: WindowType) {
        let window_state = self.get_state(&window_type);
        let window_struct = Window {
            id: window,
            workspace: self.current_workspace,
            window_type,
            window_state,
        };

        self.workspaces[self.current_workspace] // Add to workspace before mapping so if MapWindow fails,
            .windows // we still know about it
            .push(window_struct);

        // dock windows shouldn't get borders
        self.conn.send_request(&xcb::x::ConfigureWindow {
            window,
            value_list: &[xcb::x::ConfigWindow::BorderWidth(2)],
        });

        self.conn.send_request(&xcb::x::ChangeWindowAttributes {
            window,
            value_list: &[xcb::x::Cw::BorderPixel(self.config.border_unfocused)],
        });
        self.clients.push(window_struct); // global client list for EWMH
    }

    fn handle_dock(&mut self, window: xcb::x::Window) {
        let cookie = self.conn.send_request(&xcb::x::GetProperty {
            delete: false,
            window,
            property: self.ewmh.atoms.net_wm_strut_partial,
            r#type: xcb::x::ATOM_CARDINAL,
            long_offset: 0,
            long_length: 12,
        });

        let reply = self.conn.wait_for_reply(cookie).unwrap();
        let values: &[u32] = reply.value();

        let strut = Strut {
            left: values[0],
            right: values[1],
            top: values[2],
            bottom: values[3],
        };

        self.docks.push(Dock { id: window, strut });
    }
}
