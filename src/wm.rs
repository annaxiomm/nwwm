use xcb::{self, x};

pub struct WindowManager {
    conn: xcb::Connection,
}

impl WindowManager {
    pub fn new() -> Self {
        let (conn, _) = xcb::Connection::connect(None).expect(
            "[nwwm] failed to connect to your display. check your DISPLAY environment variable.",
        );

        Self { conn: conn }
    }

    pub fn run(&self) {
        let screen = self
            .conn
            .get_setup()
            .roots()
            .next()
            .expect("[nwwm] failed to get your screen.");
        let root_window = screen.root();

        let cookie = self.conn.send_request_checked(&x::ChangeWindowAttributes {
            window: root_window,
            value_list: &[x::Cw::EventMask(
                x::EventMask::SUBSTRUCTURE_NOTIFY
                    | x::EventMask::SUBSTRUCTURE_REDIRECT
                    | x::EventMask::KEY_PRESS,
            )],
        });

        self.conn
            .check_request(cookie)
            .expect("[nwwm] failed to initialise nwwm. is another wm running?");

        loop {
            match self.conn.wait_for_event() {
                Ok(event) => match event {
                    xcb::Event::X(x::Event::KeyPress(key)) => {
                        println!("Key Pressed: {}", key.detail());
                    }

                    _ => {}
                },

                Err(err) => {
                    eprintln!("[nwwm] X11 connection error {err}");
                    break;
                }
            }
        }
    }
}
