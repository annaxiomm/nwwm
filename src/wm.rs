use xcb::{self, x};

use crate::{
    err::NwwmError,
    logger::{self, LogLevel},
};

#[derive(Debug)]
pub struct Window {
    pub id: xcb::x::Window,
    pub workspace: usize,
}

pub struct Workspace {
    pub windows: Vec<Window>,
}

pub struct WindowManager {
    pub conn: xcb::Connection, // conn is public so handlers can access it from handlers.rs
    pub workspaces: Vec<Workspace>, // same here
    pub current_workspace: usize,
    logger: logger::Logger,
    screennum: i32,
}

impl WindowManager {
    pub fn new(logger: logger::Logger) -> Result<Self, NwwmError> {
        let (conn, screennum) =
            xcb::Connection::connect(None).map_err(|_| NwwmError::DisplayUnavailable)?;

        let workspaces: Vec<Workspace> = vec![Workspace {
            windows: Vec::new(),
        }];

        Ok(Self {
            conn,
            workspaces,
            logger,
            current_workspace: 0,
            screennum,
        })
    }

    pub fn run(&mut self) -> Result<(), NwwmError> {
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
                        self.logger.log("creating window...", LogLevel::Debug);
                        self.on_map_request(event);
                    }
                    xcb::Event::X(x::Event::DestroyNotify(event)) => {
                        self.on_destroy_notify(event);
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
