use crate::{
    keybinds::{Action, Keybind},
    tile::Layout,
};
use xkbcommon::xkb;

#[allow(dead_code)]
pub struct Config {
    pub border_width: u32,
    pub border_focused: u32,
    pub border_unfocused: u32,
    pub mod_key: xcb::x::ModMask,
    pub keybinds: Vec<Keybind>,
    pub startup: Vec<String>,
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

        let mod_key = xcb::x::ModMask::N4;
        let keybinds = vec![
            Keybind {
                modifiers: mod_key,
                keysym: xkb::keysyms::KEY_w,
                action: Action::FocusNext,
            },
            Keybind {
                modifiers: mod_key,
                keysym: xkb::keysyms::KEY_m,
                action: Action::SetLayout(Layout::Monocle),
            },
            Keybind {
                modifiers: mod_key,
                keysym: xkb::keysyms::KEY_s,
                action: Action::SetLayout(Layout::MasterStack),
            },
            Keybind {
                modifiers: mod_key,
                keysym: xkb::keysyms::KEY_c,
                action: Action::CloseWindow,
            },
            Keybind {
                modifiers: mod_key,
                keysym: xkb::keysyms::KEY_1,
                action: Action::SwitchWorkspace(1),
            },
            Keybind {
                modifiers: mod_key,
                keysym: xkb::keysyms::KEY_2,
                action: Action::SwitchWorkspace(2),
            },
            Keybind {
                modifiers: mod_key,
                keysym: xkb::keysyms::KEY_3,
                action: Action::SwitchWorkspace(3),
            },
            Keybind {
                modifiers: mod_key,
                keysym: xkb::keysyms::KEY_4,
                action: Action::SwitchWorkspace(4),
            },
            Keybind {
                modifiers: mod_key,
                keysym: xkb::keysyms::KEY_5,
                action: Action::SwitchWorkspace(5),
            },
            Keybind {
                modifiers: mod_key,
                keysym: xkb::keysyms::KEY_r,
                action: Action::Exec("rofi -show run".to_string()),
            },
            Keybind {
                modifiers: xcb::x::ModMask::N4,
                keysym: xkb::keysyms::KEY_Return,
                action: Action::Exec("kitty".to_string()),
            },
            Keybind {
                modifiers: mod_key | xcb::x::ModMask::SHIFT,
                keysym: xkb::keysyms::KEY_q,
                action: Action::Quit,
            },
            Keybind {
                modifiers: xcb::x::ModMask::N4,
                keysym: xkb::keysyms::KEY_n,
                action: Action::Exec("test-unavailable-command".to_string()), // test keybind to test trying to open non-existent programs
            },
        ];
        let startup: Vec<String> = vec![
            "feh --bg-fill ~/Pictures/Wallpapers/blahaj.png".to_string(),
            "polybar".to_string(),
        ];
        Self {
            border_width,
            border_focused,
            border_unfocused,
            mod_key,
            keybinds,
            startup,
        }
    }
}
