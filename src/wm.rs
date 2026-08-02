use xcb::{self, x};

use crate::err::NwwmError;

pub struct WindowManager {
    conn: xcb::Connection,
}

impl WindowManager {
    pub fn new() -> Result<Self, NwwmError> {
        let (conn, _) =
            xcb::Connection::connect(None).map_err(|_| NwwmError::DisplayUnavailable)?;

        Ok(Self { conn: conn })
    }

    pub fn run(&self) -> Result<(), NwwmError> {
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
            .map_err(|_| NwwmError::InitError)?;

        loop {
            match self.conn.wait_for_event() {
                Ok(event) => match event {
                    xcb::Event::X(x::Event::KeyPress(key)) => {
                        println!("Key Pressed: {}", key.detail());
                    }

                    _ => {}
                },

                Err(_err) => {
                    return Err(NwwmError::XCBConnError);
                }
            }
        }
    }
}
