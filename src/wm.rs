use xcb::{self, x};

use crate::{
    err::NwwmError,
    logger::{self, LogLevel},
};

pub struct WindowManager {
    conn: xcb::Connection,
    logger: logger::Logger,
    screennum: i32,
}

impl WindowManager {
    pub fn new(logger: logger::Logger) -> Result<Self, NwwmError> {
        let (conn, screennum) =
            xcb::Connection::connect(None).map_err(|_| NwwmError::DisplayUnavailable)?;

        Ok(Self {
            conn,
            logger,
            screennum,
        })
    }

    pub fn run(&self) -> Result<(), NwwmError> {
        let screen = self
            .conn
            .get_setup()
            .roots()
            .nth(self.screennum as usize)
            .ok_or(NwwmError::ScreenGrabError)
            .unwrap();
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
            .map_err(|_| NwwmError::InitError)?; // If the cookie rejects, we aren't the wm

        loop {
            match self.conn.wait_for_event() {
                Ok(event) => match event {
                    xcb::Event::X(x::Event::KeyPress(key)) => {
                        self.logger.log(
                            format!("key pressed: {:?}", key.detail()).as_str(),
                            logger::LogLevel::Info,
                        );
                    }
                    xcb::Event::X(x::Event::MapRequest(event)) => {
                        self.logger.log("creating window", LogLevel::Debug);
                        self.create_window(event);
                    }

                    _ => {
                        println!("{:?}", event)
                    }
                },

                Err(_err) => {
                    return Err(NwwmError::XCBConnError);
                }
            }
        }
    }

    fn create_window(&self, event: xcb::x::MapRequestEvent) {
        self.conn.send_request(&xcb::x::MapWindow {
            window: event.window(),
        });

        self.conn.flush().unwrap();
    }
}
