use std::collections::HashMap;

use xcb::{self, x};

use crate::{
    atoms::Atoms,
    config::Config,
    err::NwwmError,
    ewmh::Ewmh,
    logger::{self},
    tile::{self, Layout, LayoutParams},
};

#[derive(Debug)]
#[allow(dead_code)]
pub enum WindowType {
    Dialog,
    Dock,
    Normal,
}

pub enum WindowState {
    Tiled,
    Floating,
}

#[allow(dead_code)]
pub struct Window {
    pub id: xcb::x::Window,
    pub workspace: usize,
    pub window_type: WindowType,
    pub window_state: WindowState,
}

pub struct Workspace {
    pub windows: Vec<Window>,
    pub layout: Layout,
}

pub struct WindowManager {
    pub conn: xcb::Connection, // conn is public so handlers can access it from handlers.rs
    pub workspaces: Vec<Workspace>, // same here
    pub ewmh: Ewmh,
    pub config: Config,
    pub current_workspace: usize,
    pub focused: Option<xcb::x::Window>,
    logger: logger::Logger,
    screennum: i32,
}

impl WindowManager {
    pub fn new(logger: logger::Logger) -> Result<Self, NwwmError> {
        let (conn, screennum) =
            xcb::Connection::connect(None).map_err(|_| NwwmError::DisplayUnavailable)?;

        let screen = conn
            .get_setup()
            .roots()
            .nth(screennum as usize)
            .ok_or(NwwmError::ScreenGrabError)
            .unwrap();

        let root_window = screen.root();

        let atoms = Atoms::new(&conn).map_err(|_| NwwmError::InitError)?;
        let ewmh = Ewmh::new(atoms, &conn, root_window).map_err(|_| NwwmError::InitError)?;
        ewmh.setup(&conn);

        let config = Config::new(&conn, &screen);

        let workspaces: Vec<Workspace> = vec![Workspace {
            windows: Vec::new(),
            layout: Layout::Monocle,
        }];

        Ok(Self {
            conn,
            workspaces,
            ewmh,
            config,
            logger,
            focused: None,
            current_workspace: 0,
            screennum,
        })
    }

    pub fn run(&mut self) -> Result<(), NwwmError> {
        self.check_other_wm(self.ewmh.root)?;

        self.grab_keys();

        loop {
            match self.conn.wait_for_event() {
                Ok(event) => match event {
                    xcb::Event::X(x::Event::KeyPress(key)) => {
                        self.on_key_press(key)?;
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
            .filter(|w| matches!(w.window_state, WindowState::Tiled))
            .map(|w| w.id)
            .collect();

        let tile_layout: HashMap<x::Window, LayoutParams> =
            match self.workspaces[self.current_workspace].layout {
                Layout::Columns => tile::columns(
                    screen.height_in_pixels(),
                    screen.width_in_pixels(),
                    windows,
                    &self.config,
                )?,
                Layout::Monocle => tile::monocle(
                    screen.height_in_pixels(),
                    screen.width_in_pixels(),
                    windows,
                    &self.config,
                )?,
            };

        for (window, param) in &tile_layout {
            self.move_window(window, param.x, param.y)?;
            self.resize_window(window, param.width, param.height)?;
        }

        self.conn.flush().unwrap();

        Ok(())
    }

    pub fn focus_window(&mut self, window: xcb::x::Window) -> Result<(), NwwmError> {
        if self.focused == Some(window) {
            return Ok(());
        }

        self.conn.send_request(&xcb::x::SetInputFocus {
            revert_to: x::InputFocus::PointerRoot,
            focus: window,
            time: xcb::x::CURRENT_TIME,
        });

        self.conn.send_request(&xcb::x::ConfigureWindow {
            window,
            value_list: &[xcb::x::ConfigWindow::StackMode(xcb::x::StackMode::Above)],
        });

        self.conn.send_request(&xcb::x::ChangeWindowAttributes {
            window,
            value_list: &[xcb::x::Cw::BorderPixel(self.config.border_focused)],
        });

        if let Some(old) = self.focused {
            self.conn.send_request(&xcb::x::ChangeWindowAttributes {
                window: old,
                value_list: &[xcb::x::Cw::BorderPixel(self.config.border_unfocused)],
            });
        }

        self.focused = Some(window);

        self.conn.flush().unwrap();

        Ok(())
    }

    pub fn focus_next(&mut self) -> Result<(), NwwmError> {
        let next = {
            let workspace = &self.workspaces[self.current_workspace];
            if workspace.windows.is_empty() {
                return Ok(());
            }

            match self.focused {
                Some(current) => {
                    let current_index = workspace
                        .windows
                        .iter()
                        .position(|w| w.id == current)
                        .unwrap_or(0);
                    workspace.windows[(current_index + 1) % workspace.windows.len()].id
                }

                None => workspace.windows[0].id,
            }
        };

        self.focus_window(next)?;

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

    fn get_window(&self, id: x::Window) -> Option<&Window> {
        self.workspaces
            .iter()
            .flat_map(|ws| ws.windows.iter())
            .find(|w| w.id == id)
    }
}
