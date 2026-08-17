use crate::{atoms::Atoms, err::NwwmError, wm::Window};

#[allow(dead_code)]
pub struct Ewmh {
    pub atoms: Atoms,
    pub root: xcb::x::Window,
    check_window: xcb::x::Window,
}

impl Ewmh {
    pub fn new(
        atoms: Atoms,
        conn: &xcb::Connection,
        root: xcb::x::Window,
    ) -> Result<Self, NwwmError> {
        let check_window = conn.generate_id();
        conn.send_request(&xcb::x::CreateWindow {
            depth: xcb::x::COPY_FROM_PARENT as u8,
            wid: check_window,
            parent: root,
            x: 0,
            y: 0,
            width: 1,
            height: 1,
            border_width: 0,
            class: xcb::x::WindowClass::InputOutput,
            visual: xcb::x::COPY_FROM_PARENT,
            value_list: &[],
        });

        Ok(Self {
            atoms,
            root,
            check_window,
        })
    }

    pub fn setup(&self, conn: &xcb::Connection) {
        conn.send_request(&xcb::x::ChangeProperty {
            mode: xcb::x::PropMode::Replace,
            window: self.root,
            property: self.atoms.net_supporting_wm_check,
            r#type: xcb::x::ATOM_WINDOW,
            data: &[self.check_window],
        });
        conn.send_request(&xcb::x::ChangeProperty {
            mode: xcb::x::PropMode::Replace,
            window: self.check_window,
            property: self.atoms.net_supporting_wm_check,
            r#type: xcb::x::ATOM_WINDOW,
            data: &[self.check_window],
        });
        conn.send_request(&xcb::x::ChangeProperty {
            mode: xcb::x::PropMode::Replace,
            window: self.check_window,
            property: self.atoms.net_wm_name,
            r#type: self.atoms.utf8_string,
            data: b"nwwm",
        });

        conn.send_request(&xcb::x::ChangeProperty {
            mode: xcb::x::PropMode::Replace,
            window: self.root,
            property: self.atoms.net_supported,
            r#type: xcb::x::ATOM_ATOM,
            data: &[
                self.atoms.net_supporting_wm_check,
                self.atoms.net_wm_name,
                self.atoms.net_client_list,
            ],
        });
    }

    pub fn update_client_list(&self, conn: &xcb::Connection, client_list: &[Window]) {
        let windows: Vec<xcb::x::Window> = client_list.iter().map(|w| w.id).collect();
        conn.send_request(&xcb::x::ChangeProperty {
            mode: xcb::x::PropMode::Replace,
            window: self.root,
            property: self.atoms.net_client_list,
            r#type: xcb::x::ATOM_WINDOW,
            data: &windows,
        });
    }
}
