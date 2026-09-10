use serde::Deserialize;
use std::{fs, io::Write};

use crate::{
    err::NwwmError,
    keybinds::{Action, Keybind},
    logger::{LogLevel, Logger},
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

#[derive(Deserialize, Debug)]
pub struct FileConfig {
    border_width: u32,
    border_focused: String,
    border_unfocused: String,
    mod_key: String,
    startup: Vec<String>,
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

// prefer to use american spelling (color) in code,
// british spelling (colour) can be used elsewhere if wanted
fn parse_color(color: &str) -> Result<(u16, u16, u16), NwwmError> {
    let mut hex = color;
    if color.starts_with("#") {
        hex = color.strip_prefix("#").unwrap(); // colours can start with # or not
    }

    if hex.len() != 6 {
        return Err(NwwmError::HexFormatError);
    }

    let r = u8::from_str_radix(&hex[0..2], 16).map_err(|_| NwwmError::HexFormatError)?;
    let g = u8::from_str_radix(&hex[2..4], 16).map_err(|_| NwwmError::HexFormatError)?;
    let b = u8::from_str_radix(&hex[4..6], 16).map_err(|_| NwwmError::HexFormatError)?;

    Ok((r as u16 * 0x0101, g as u16 * 0x0101, b as u16 * 0x0101))
}

fn alloc_color_from_hex(
    logger: &Logger,
    conn: &xcb::Connection,
    screen: &xcb::x::Screen,
    hex: &str,
) -> u32 {
    let hex_parsed = parse_color(hex);
    if hex_parsed.is_err() {
        logger.log(
            format!("unable to parse colour \"{}\"colours should be in #RRGGBB format. defaulting to black", hex).as_str(),
            LogLevel::Error,
        );
        return alloc_color(conn, screen, 0x0000, 0x0000, 0x0000);
    }

    let hex_values = hex_parsed.unwrap();

    alloc_color(conn, screen, hex_values.0, hex_values.1, hex_values.2)
}

fn load_config(logger: &Logger) -> Option<FileConfig> {
    let default_config = include_str!("../config/default.toml");

    let config_dir = dirs::config_dir()
        .expect("Could not find config dir!")
        .join("nwwm");

    if fs::create_dir_all(&config_dir).is_err() {
        logger.log("failed to create config directory", LogLevel::Error);
        return None;
    }

    let config_file = config_dir.join("config.toml");

    if !config_file.exists() {
        logger.log(
            "no config file was found! creating one using defaults...",
            LogLevel::Info,
        );

        match fs::File::create_new(&config_file) {
            Ok(mut file) => {
                if file.write_all(default_config.as_bytes()).is_err() {
                    logger.log("failed to write config file", LogLevel::Error);
                }
            }
            Err(_) => {
                logger.log("failed to create config file", LogLevel::Error);
            }
        }
    }

    let config_contents = fs::read_to_string(config_file).unwrap();
    let config: FileConfig = toml::from_str(&config_contents).unwrap();

    Some(config)
}

impl Config {
    pub fn new(conn: &xcb::Connection, screen: &xcb::x::Screen, logger: &Logger) -> Self {
        let config_file = load_config(logger).unwrap();

        let border_width = config_file.border_width;

        let border_focused =
            alloc_color_from_hex(logger, conn, screen, config_file.border_focused.as_str());
        let border_unfocused =
            alloc_color_from_hex(logger, conn, screen, config_file.border_unfocused.as_str());
        let mod_key = match config_file.mod_key.as_str() {
            "Mod1" => xcb::x::ModMask::N1,
            "Mod2" => xcb::x::ModMask::N2,
            "Mod3" => xcb::x::ModMask::N3,
            "Mod4" => xcb::x::ModMask::N4,
            "Mod5" => xcb::x::ModMask::N5,
            _ => {
                logger.log("invalid \"mod_key\" in config, using Mod4", LogLevel::Error);
                xcb::x::ModMask::N4
            }
        };

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
                modifiers: mod_key,
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
        let startup: Vec<String> = config_file.startup;

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
