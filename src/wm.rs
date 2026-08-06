use std::collections::HashMap;

use xcb::{self, x};

use crate::{
    err::NwwmError,
    logger::{self},
    tile::{self, Layout, LayoutParams},
};

#[derive(Debug)]
#[allow(dead_code)]
pub struct Window {
    pub id: xcb::x::Window,
    pub workspace: usize,
}

pub struct Workspace {
    pub windows: Vec<Window>,
    pub layout: Layout,
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
            layout: Layout::BasicTile,
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

        self.check_other_wm(root_window)?;

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
                        self.on_map_request(event)?;
                    }
                    xcb::Event::X(x::Event::DestroyNotify(event)) => {
                        self.on_destroy_notify(event)?;
                    }
                    xcb::Event::X(x::Event::ButtonPress(event)) => {
                        self.on_button_press(event)?;
                    }

                    _ => {}
                },

                Err(err) => {
                    println!("{err}");
                    return Err(NwwmError::XCBConnError);
                }
            }
        }
    }

    pub fn tile(&mut self) -> Result<(), NwwmError> {
        println!("tiling!");

        let screen = self
            .conn
            .get_setup()
            .roots()
            .nth(self.screennum as usize)
            .ok_or(NwwmError::ScreenGrabError)
            .unwrap();
        let windows: Vec<xcb::x::Window> = self.workspaces[self.current_workspace]
            .windows
            .iter()
            .map(|w| w.id)
            .collect();

        println!("windows to be tiled: {:?}", windows);

        let tile_layout: HashMap<x::Window, LayoutParams> =
            match self.workspaces[self.current_workspace].layout {
                Layout::BasicTile => {
                    tile::basic(screen.height_in_pixels(), screen.width_in_pixels(), windows)?
                }
            };

        for (window, param) in &tile_layout {
            self.move_window(window, param.x, param.y)?;
            self.resize_window(window, param.width, param.height)?;
        }

        self.conn.flush().unwrap();

        Ok(())
    }

    pub fn focus_window(&mut self, window: xcb::x::Window) -> Result<(), NwwmError> {
        println!("{:?}", window);

        self.conn.send_request(&xcb::x::SetInputFocus {
            revert_to: xcb::x::InputFocus::PointerRoot,
            focus: window,
            time: xcb::x::CURRENT_TIME,
        });

        self.conn.flush().unwrap();

        Ok(())
    }

    fn check_other_wm(&self, root: xcb::x::Window) -> Result<(), NwwmError> {
        let cookie = self.conn.send_request_checked(&x::ChangeWindowAttributes {
            window: root,
            value_list: &[x::Cw::EventMask(
                x::EventMask::SUBSTRUCTURE_NOTIFY
                    | x::EventMask::SUBSTRUCTURE_REDIRECT
                    | x::EventMask::KEY_PRESS,
            )],
        });

        self.conn
            .check_request(cookie)
            .map_err(|_| NwwmError::InitError)?; // If the cookie rejects, we aren't the wm

        Ok(())
    }

    fn move_window(&self, window: &x::Window, x: i32, y: i32) -> Result<(), NwwmError> {
        self.conn.send_request(&xcb::x::ConfigureWindow {
            window: *window,
            value_list: &[xcb::x::ConfigWindow::X(x), xcb::x::ConfigWindow::Y(y)],
        });

        Ok(())
    }

    fn resize_window(&self, window: &x::Window, width: u32, height: u32) -> Result<(), NwwmError> {
        self.conn.send_request(&xcb::x::ConfigureWindow {
            window: *window,
            value_list: &[
                xcb::x::ConfigWindow::Width(width),
                xcb::x::ConfigWindow::Height(height),
            ],
        });

        Ok(())
    }
}
