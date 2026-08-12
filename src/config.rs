use crate::{
    keybinds::{Action, Keybind},
    tile::Layout,
};
use xkbcommon::xkb;

pub struct Config {
    pub border_width: u32,
    pub border_focused: u32,
    pub border_unfocused: u32,
    pub keybinds: Vec<Keybind>,
}

fn alloc_color(
    conn: &xcb::Connection,
    screen: &xcb::x::Screen,
    red: u16,
    green: u16,
    blue: u16,
) -> u32 {
    let cookie = conn.send_request(&xcb::x::AllocColor {
        cmap: screen.default_colormap(),
        red,
        green,
        blue,
    });
    let reply = conn.wait_for_reply(cookie).unwrap();
    reply.pixel()
}

impl Config {
    pub fn new(conn: &xcb::Connection, screen: &xcb::x::Screen) -> Self {
        let border_width: u32 = 2;
        let border_focused = alloc_color(conn, screen, 0xffff, 0, 0);
        let border_unfocused = alloc_color(conn, screen, 0xffff, 0xffff, 0xffff);
        let keybinds = vec![
            Keybind {
                modifiers: xcb::x::ModMask::N4,
                keysym: xkb::keysyms::KEY_w,
                action: Action::FocusNext,
            },
            Keybind {
                modifiers: xcb::x::ModMask::N4,
                keysym: xkb::keysyms::KEY_m,
                action: Action::SetLayout(Layout::Monocle),
            },
            Keybind {
                modifiers: xcb::x::ModMask::N4,
                keysym: xkb::keysyms::KEY_s,
                action: Action::SetLayout(Layout::MasterStack),
            },
            Keybind {
                modifiers: xcb::x::ModMask::N4,
                keysym: xkb::keysyms::KEY_Return,
                action: Action::Exec("kitty".to_string()),
            },
        ];
        Self {
            border_width,
            border_focused,
            border_unfocused,
            keybinds,
        }
    }
}
