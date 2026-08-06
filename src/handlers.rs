// HANDLERS.RS
// -----------
// event handlers go here to avoid clogging up wm.rs

use crate::{
    err::NwwmError,
    wm::{Window, WindowManager},
};

impl WindowManager {
    pub fn on_map_request(&mut self, ev: xcb::x::MapRequestEvent) -> Result<(), NwwmError> {
        let window = ev.window();

        self.workspaces[self.current_workspace] // Add to workspace before mapping so if MapWindow fails,
            .windows // we still know about it
            .push(Window {
                id: window,
                workspace: self.current_workspace,
            });

        self.conn.send_request(&xcb::x::MapWindow { window });

        self.conn.send_request(&xcb::x::SetInputFocus {
            revert_to: xcb::x::InputFocus::PointerRoot,
            focus: window,
            time: xcb::x::CURRENT_TIME,
        });

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

        // test configure
        self.conn.send_request(&xcb::x::ConfigureWindow {
            window,
            value_list: &[
                xcb::x::ConfigWindow::Width(400),
                xcb::x::ConfigWindow::Height(300),
            ],
        });

        self.tile()?;

        self.conn.flush().unwrap(); // without this, nothing happens

        Ok(())
    }

    pub fn on_destroy_notify(&mut self, ev: xcb::x::DestroyNotifyEvent) -> Result<(), NwwmError> {
        let window = ev.window();

        for workspace in &mut self.workspaces {
            workspace.windows.retain(|w| w.id != window);
        }

        self.tile()?;

        Ok(())
    }

    pub fn on_button_press(&mut self, ev: xcb::x::ButtonPressEvent) -> Result<(), NwwmError> {
        println!("focusing window: {:?}", ev.event());

        self.focus_window(ev.event())?;

        Ok(())
    }
}
