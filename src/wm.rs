// |-----------|
// | WM.RS     |
// |-----------|
// Contains most

use std::collections::HashMap;

use xcb::{self, x};
use xkbcommon::xkb;

use crate::{
    atoms::Atoms,
    config::Config,
    err::NwwmError,
    ewmh::Ewmh,
    logger::{self, LogLevel},
    tile::{self, Layout},
};

#[derive(Debug, Clone, Copy)]
#[allow(dead_code)]
pub enum WindowType {
    Dialog,
    Dock,
    Utility,
    Normal,
}

#[derive(Clone, Copy)]
pub enum WindowState {
    Tiled,
    Floating,
}

pub struct Rect {
    pub x: i32,
    pub y: i32,
    pub width: u32,
    pub height: u32,
}

#[derive(Clone, Copy)]
#[allow(dead_code)]
pub struct Window {
    pub id: xcb::x::Window,
    pub workspace: usize,
    pub window_type: WindowType,
    pub window_state: WindowState,
}

pub struct Strut {
    pub left: u32,
    pub right: u32,
    pub top: u32,
    pub bottom: u32,
}

pub struct Dock {
    pub id: xcb::x::Window,
    pub strut: Strut,
}

#[derive(Clone)]
pub struct Workspace {
    pub windows: Vec<Window>,
    pub layout: Layout,
    pub focused: Option<xcb::x::Window>,
}

pub struct WindowManager {
    pub conn: xcb::Connection,
    pub workspaces: Vec<Workspace>,
    pub num_workspaces: usize,
    pub clients: Vec<Window>, // list of all managed windows regardless of workspace
    pub docks: Vec<Dock>,
    pub ewmh: Ewmh,
    pub config: Config,
    pub current_workspace: usize,
    pub focused: Option<xcb::x::Window>,
    pub xkb_state: xkb::State,
    pub xkb_keymap: xkb::Keymap,
    pub logger: logger::Logger,
    pub screen_area: Rect,
    pub should_quit: bool,
    _screennum: i32,
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

        // select mousedown events for focusing
        conn.send_request(&xcb::x::GrabButton {
            owner_events: false,
            grab_window: root_window,
            event_mask: xcb::x::EventMask::BUTTON_PRESS,
            pointer_mode: xcb::x::GrabMode::Sync,
            keyboard_mode: xcb::x::GrabMode::Async,
            confine_to: xcb::x::WINDOW_NONE,
            cursor: xcb::x::CURSOR_NONE,
            button: xcb::x::ButtonIndex::N1,
            modifiers: xcb::x::ModMask::ANY,
        });

        logger.log("initialising EWMH system...", LogLevel::Debug);
        let atoms = Atoms::new(&conn).map_err(|_| NwwmError::InitError)?;
        let ewmh = Ewmh::new(atoms, &conn, root_window).map_err(|_| NwwmError::InitError)?;
        ewmh.setup(&conn);

        logger.log("initialisting config...", LogLevel::Debug);
        let config = Config::new(&conn, screen, &logger);

        let num_workspaces: usize = 5;

        let workspaces: Vec<Workspace> = vec![
            Workspace {
                windows: Vec::new(),
                layout: config.default_layout,
                focused: None
            };
            5
        ];

        let clients: Vec<Window> = Vec::new();
        let docks: Vec<Dock> = Vec::new();

        logger.log("initialising xkb...", LogLevel::Debug);
        let context = xkb::Context::new(xkb::COMPILE_NO_FLAGS);
        let xkb_keymap = xkb::Keymap::new_from_names(
            &context,
            "",
            "",
            "",
            "",
            None,
            xkb::KEYMAP_COMPILE_NO_FLAGS,
        )
        .ok_or(NwwmError::XKBError)?;
        let xkb_state = xkb::State::new(&xkb_keymap);

        let screen_area = Rect {
            x: 0,
            y: 0,
            width: screen.width_in_pixels() as u32,
            height: screen.height_in_pixels() as u32,
        };

        Ok(Self {
            conn,
            workspaces,
            num_workspaces,
            clients,
            docks,
            ewmh,
            config,
            logger,
            focused: None,
            xkb_state,
            xkb_keymap,
            current_workspace: 0,
            screen_area,
            should_quit: false,
            _screennum: screennum,
        })
    }

    pub fn run(&mut self) -> Result<(), NwwmError> {
        self.logger.log(
            "checking for other running window managers...",
            LogLevel::Debug,
        );
        self.check_other_wm(self.ewmh.root)?;

        self.grab_keys();

        // focus root window so keypress events will be detected
        self.conn.send_request(&xcb::x::SetInputFocus {
            revert_to: xcb::x::InputFocus::PointerRoot,
            focus: self.ewmh.root,
            time: xcb::x::CURRENT_TIME,
        });
        self.conn.flush().unwrap();

        self.ewmh
            .update_number_of_desktops(&self.conn, self.num_workspaces as u32);

        self.run_startup_cmds()?;

        loop {
            match self.conn.wait_for_event() {
                Ok(event) => match event {
                    xcb::Event::X(x::Event::KeyPress(key)) => {
                        self.on_key_press(key)?;
                    }
                    xcb::Event::X(x::Event::MapRequest(event)) => {
                        self.on_map_request(event)?;
                    }
                    xcb::Event::X(x::Event::ConfigureRequest(event)) => {
                        self.on_config_request(event)?;
                    }
                    xcb::Event::X(x::Event::DestroyNotify(event)) => {
                        self.on_destroy_notify(event)?;
                    }
                    xcb::Event::X(x::Event::ButtonPress(event)) => {
                        self.on_button_press(event)?;
                    }
                    xcb::Event::X(x::Event::ClientMessage(event)) => {
                        self.on_client_message(event)?;
                    }

                    _ => {}
                },

                Err(err) => {
                    println!("Unknown error encountered: {err:?}");
                    return Err(NwwmError::XCBConnError);
                }
            }

            if self.should_quit {
                return Ok(());
            }
        }
    }

    // tiles the current workspace according to its selected layout
    pub fn tile(&mut self) -> Result<(), NwwmError> {
        let windows: Vec<xcb::x::Window> = self.workspaces[self.current_workspace]
            .windows
            .iter()
            .filter(|w| matches!(w.window_state, WindowState::Tiled))
            .map(|w| w.id)
            .collect();

        let tile_layout: HashMap<x::Window, Rect> = match self.workspaces[self.current_workspace]
            .layout
        {
            Layout::Columns => tile::columns(&self.screen_area, windows, &self.config)?,
            Layout::Monocle => tile::monocle(&self.screen_area, windows, &self.config)?,
            Layout::MasterStack => tile::master_stack(&self.screen_area, windows, &self.config)?,
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

        // bring window to the top - for floating and monocle mode
        let cookie = self.conn.send_request_checked(&xcb::x::ConfigureWindow {
            window,
            value_list: &[xcb::x::ConfigWindow::StackMode(xcb::x::StackMode::Above)],
        });

        if let Err(e) = self.conn.check_request(cookie) {
            println!("ConfigureWindow failed: {e}");
        }

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
        self.workspaces[self.current_workspace].focused = Some(window);

        self.ewmh.set_active_window(&self.conn, window);

        self.conn.flush().unwrap();

        Ok(())
    }

    // sets the current
    pub fn unfocus(&mut self) {
        if let Some(old) = self.focused {
            self.conn.send_request(&xcb::x::ChangeWindowAttributes {
                window: old,
                value_list: &[xcb::x::Cw::BorderPixel(self.config.border_unfocused)],
            });
        }

        self.focused = None;
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

    pub fn close_window(&self, window: xcb::x::Window) -> Result<(), NwwmError> {
        self.conn.send_request(&xcb::x::DestroyWindow { window });
        self.conn.flush().unwrap();

        Ok(())
    }

    pub fn set_layout(&mut self, layout: Layout) -> Result<(), NwwmError> {
        self.workspaces[self.current_workspace].layout = layout;
        self.tile()?;

        Ok(())
    }

    // recalculates available screen area based on currently active dock windows
    pub fn recalculate_screen_area(&mut self) {
        let screen = self
            .conn
            .get_setup()
            .roots()
            .nth(self._screennum as usize)
            .ok_or(NwwmError::ScreenGrabError)
            .unwrap();

        // get reserved area from all active docks
        let left = self.docks.iter().map(|d| d.strut.left).max().unwrap_or(0);
        let right = self.docks.iter().map(|d| d.strut.right).max().unwrap_or(0);
        let top = self.docks.iter().map(|d| d.strut.top).max().unwrap_or(0);
        let bottom = self.docks.iter().map(|d| d.strut.bottom).max().unwrap_or(0);

        self.screen_area = Rect {
            x: left as i32,
            y: top as i32,
            width: screen.width_in_pixels() as u32 - (left + right),
            height: screen.height_in_pixels() as u32 - (top + bottom),
        };
    }

    // selects events that nwwm wants to listen to,
    // if this request fails then 9/10 times another wm is running
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
        let cookie = self.conn.send_request_checked(&xcb::x::ConfigureWindow {
            window: *window,
            value_list: &[xcb::x::ConfigWindow::X(x), xcb::x::ConfigWindow::Y(y)],
        });

        if let Err(e) = self.conn.check_request(cookie) {
            println!("ConfigureWindow failed: {:?}", e);
        }

        Ok(())
    }

    fn resize_window(&self, window: &x::Window, width: u32, height: u32) -> Result<(), NwwmError> {
        let cookie = self.conn.send_request_checked(&xcb::x::ConfigureWindow {
            window: *window,
            value_list: &[
                xcb::x::ConfigWindow::Width(width),
                xcb::x::ConfigWindow::Height(height),
            ],
        });

        if let Err(e) = self.conn.check_request(cookie) {
            println!("ConfigureWindow failed: {:?}", e);
        }

        Ok(())
    }

    // probably should make this a bit more fleshed out
    pub fn quit(&mut self) {
        self.logger.log("goodbye !", LogLevel::Info);
        self.should_quit = true;
    }

    // i don't know why this function exists but
    // it can stay i guess - finds an nwwm Window from
    // its x11 id
    fn _get_window(&self, id: x::Window) -> Option<&Window> {
        self.workspaces
            .iter()
            .flat_map(|ws| ws.windows.iter())
            .find(|w| w.id == id)
    }

    fn run_startup_cmds(&self) -> Result<(), NwwmError> {
        self.logger
            .log("running startup commands...", LogLevel::Debug);
        for cmd in &self.config.startup {
            self.logger.log(cmd.as_str(), LogLevel::Debug);
            self.exec_command(cmd)?;
        }

        Ok(())
    }
}
