pub struct Config {
    pub border_width: u32,
    pub border_focused: u32,
    pub border_unfocused: u32,
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
        Self {
            border_width,
            border_focused,
            border_unfocused,
        }
    }
}
