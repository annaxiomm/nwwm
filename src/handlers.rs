// HANDLERS.RS
// -----------
// event handlers go here to avoid clogging up wm.rs

use crate::wm::{Window, WindowManager};

impl WindowManager {
    pub fn on_map_request(&mut self, ev: xcb::x::MapRequestEvent) {
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

        // test configure
        self.conn.send_request(&xcb::x::ConfigureWindow {
            window,
            value_list: &[
                xcb::x::ConfigWindow::Width(400),
                xcb::x::ConfigWindow::Height(300),
            ],
        });

        println!("{:?}", self.workspaces[self.current_workspace].windows);

        self.conn.flush().unwrap(); // without this, nothing happens
    }

    pub fn on_destroy_notify(&mut self, ev: xcb::x::DestroyNotifyEvent) {
        let window = ev.window();

        println!("deleting window {:?}", window);

        for workspace in &mut self.workspaces {
            workspace.windows.retain(|w| w.id != window);
        }

        println!("{:?}", self.workspaces[self.current_workspace].windows);
    }
}
