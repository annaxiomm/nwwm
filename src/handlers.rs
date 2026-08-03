// HANDLERS.RS
// -----------
// event handlers go here to avoid clogging up wm.rs

use crate::wm::WindowManager;

impl WindowManager {
    // Generally fires when
    pub fn on_map_request(&self, ev: xcb::x::MapRequestEvent) {
        self.conn.send_request(&xcb::x::MapWindow {
            window: ev.window(),
        });

        self.conn.flush().unwrap(); // without this, nothing happens
    }
}
