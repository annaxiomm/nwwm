use xcb::x;

pub fn intern_atom(conn: &xcb::Connection, name: &str) -> Result<x::Atom, xcb::Error> {
    let cookie = conn.send_request(&x::InternAtom {
        only_if_exists: false,
        name: name.as_bytes(),
    });

    let reply = conn.wait_for_reply(cookie)?;

    Ok(reply.atom())
}

pub struct Atoms {
    pub net_supported: x::Atom,
    pub net_supporting_wm_check: x::Atom,
    pub net_wm_name: x::Atom,
    pub net_wm_window_type: x::Atom,
    pub net_wm_window_type_dialog: x::Atom,
    pub net_wm_window_type_dock: x::Atom,
    pub net_wm_window_type_utility: x::Atom,

    pub utf8_string: x::Atom,
}

impl Atoms {
    pub fn new(conn: &xcb::Connection) -> Result<Self, xcb::Error> {
        Ok(Self {
            net_supported: intern_atom(conn, "_NET_SUPPORTED")?,
            net_supporting_wm_check: intern_atom(conn, "_NET_SUPPORTING_WM_CHECK")?,
            net_wm_name: intern_atom(conn, "_NET_WM_NAME")?,
            net_wm_window_type: intern_atom(conn, "_NET_WM_WINDOW_TYPE")?,
            net_wm_window_type_dialog: intern_atom(conn, "_NET_WM_WINDOW_TYPE_DIALOG")?,
            net_wm_window_type_dock: intern_atom(conn, "_NET_WM_WINDOW_TYPE_DOCK")?,
            net_wm_window_type_utility: intern_atom(conn, "_NET_WM_WINDOW_TYPE_UTILITY")?,
            utf8_string: intern_atom(conn, "UTF8_STRING")?,
        })
    }
}
