// CONFIG.rs
// ---------
// all things config, config loading, and config parsing
// should be split into separate files

use serde::Deserialize;
use std::{collections::HashMap, fs, io::Write};
use xcb::x::ModMask;

use crate::{
    err::NwwmError::{self, ParseActionError},
    keybinds::{Action, Keybind},
    logger::{LogLevel, Logger},
    tile::Layout,
};
use xkbcommon::xkb;

// the actual config that nwwm reads from
#[allow(dead_code)]
pub struct Config {
    pub border_width: u32,
    pub border_focused: u32,
    pub border_unfocused: u32,
    pub mod_key: xcb::x::ModMask,
    pub keybinds: Vec<Keybind>,
    pub startup: Vec<String>,
    pub gaps_inner: u32,
    pub gaps_outer: u32,
    pub default_layout: Layout,
}

// config parsed directly from TOML which can then
// be re-parsed into actual values useful to nwwm
#[derive(Deserialize, Debug)]
pub struct FileConfig {
    border_width: u32,
    gaps_inner: u32,
    gaps_outer: u32,
    border_focused: String,
    border_unfocused: String,
    mod_key: String,
    keybinds: HashMap<String, String>,
    startup: Vec<String>,
    default_layout: String,
}

// helper function to allocate colours for X11 - X11 doesn't
// allow you to use arbitrary colours but rather requires
// colours to be pre-allocated and referenced with a u32 id
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

    // X11 uses 48 bit colour values (#ffff00000000) instead of
    // traditional 24 bit (#ff0000) so conversion is done here by
    // multiplying by 257
    Ok((r as u16 * 0x0101, g as u16 * 0x0101, b as u16 * 0x0101))
}

fn parse_keybinds(
    mod_key: ModMask,
    keybinds: HashMap<String, String>,
) -> Result<Vec<Keybind>, NwwmError> {
    let mut keybind_list: Vec<Keybind> = Vec::new();

    for (key, value) in keybinds.into_iter() {
        let key_parsed = parse_key(mod_key, key);
        if key_parsed.is_err() {
            return Err(NwwmError::KeyBindError);
        }
        let (modifiers, keysym) = key_parsed.unwrap();

        let action_parsed = parse_action(value);
        if action_parsed.is_err() {
            return Err(NwwmError::ParseActionError);
        }
        let action = action_parsed.unwrap();

        keybind_list.push(Keybind {
            modifiers,
            keysym,
            action,
        });
    }

    Ok(keybind_list)
}

fn parse_key(mod_key: ModMask, key_string: String) -> Result<(ModMask, u32), NwwmError> {
    let key_split: Vec<&str> = key_string.split("+").collect();
    let mut modifiers = xcb::x::ModMask::empty();

    if key_split.len() < 2 {
        return Err(NwwmError::KeyBindError);
    }

    let key = key_split.last().unwrap();

    for i in &key_split[..key_split.len() - 1] {
        match i.to_lowercase().as_str() {
            "mod" => modifiers |= mod_key,
            "shift" => modifiers |= xcb::x::ModMask::SHIFT,
            "ctrl" => modifiers |= xcb::x::ModMask::CONTROL,
            "alt" => modifiers |= xcb::x::ModMask::N1,
            _ => return Err(NwwmError::ModError),
        }
    }

    let keysym = xkb::keysym_from_name(key, xkb::KEYSYM_CASE_INSENSITIVE).raw(); // KEYSYM_CASE_INSENSITIVE means that
    if keysym == xkb::keysyms::KEY_NoSymbol {
        // keysym_from_name returns KEY_NoSymbol if the key is invalid
        return Err(NwwmError::KeyBindError);
    }

    Ok((modifiers, keysym))
}

fn parse_action(action_string: String) -> Result<Action, NwwmError> {
    let action_split = match action_string.split_once(" ") {
        // split the command into an action and parameters
        Some((action_name, params)) => vec![action_name, params.trim()], // if the action has parameters (e.g. SetWorkspace)
        None => vec![action_string.as_str()], // if it's just a standalone action (e.g. Quit)
    };
    let action = match action_split[0] {
        "quit" => Action::Quit,
        "closewindow" => Action::CloseWindow,
        "exec" => {
            if action_split.len() == 1 {
                return Err(ParseActionError);
            }

            Action::Exec(action_split[1].to_string())
        }
        "setworkspace" => {
            if action_split.len() == 1 {
                return Err(ParseActionError);
            }

            let workspace = action_split[1].parse::<usize>();
            if workspace.is_err() {
                return Err(ParseActionError);
            }

            Action::SwitchWorkspace(workspace.unwrap())
        }
        "movetoworkspace" => {
            if action_split.len() == 1 {
                return Err(ParseActionError);
            }

            let workspace = action_split[1].parse::<usize>();
            if workspace.is_err() {
                return Err(ParseActionError);
            }

            Action::MoveToWorkspace(workspace.unwrap())
        }

        "setlayout" => {
            if action_split.len() == 1 {
                return Err(ParseActionError);
            }

            let layout: Option<Layout> = match action_split[1] {
                "monocle" => Some(Layout::Monocle),
                "masterstack" => Some(Layout::MasterStack),
                "columns" => Some(Layout::Columns),
                _ => None,
            };

            if layout.is_none() {
                return Err(ParseActionError);
            }

            Action::SetLayout(layout.unwrap())
        }

        "focus" => {
            if action_split.len() == 1 {
                return Err(ParseActionError);
            }

            match action_split[1] {
                "next" => Action::FocusNext,
                _ => return Err(ParseActionError),
            }
        }
        _ => Action::CloseWindow,
    };

    Ok(action)
}

fn parse_layout(l: String) -> Result<Layout, NwwmError> {
    match l.to_lowercase().as_str() {
        "monocle" => Ok(Layout::Monocle),
        "masterstack" => Ok(Layout::MasterStack),
        "columns" => Ok(Layout::Columns),
        _ => Err(NwwmError::ParseLayoutError),
    }
}

// helper function that converts a string to
// a hex colour and then allocates that as an
// X11 colour
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

        // if something goes wrong just use black
        return alloc_color(conn, screen, 0x0000, 0x0000, 0x0000);
    }

    let hex_values = hex_parsed.unwrap();

    alloc_color(conn, screen, hex_values.0, hex_values.1, hex_values.2)
}

fn load_config(logger: &Logger) -> Option<FileConfig> {
    // default config file - included at compile time
    let default_config = include_str!("../config/default.toml");

    let config_dir = dirs::config_dir()
        .expect("Could not find config dir!")
        .join("nwwm");

    // if the config dir doesn't exist, try to create one
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

        // values from the config
        let border_width = config_file.border_width;

        let border_focused =
            alloc_color_from_hex(logger, conn, screen, config_file.border_focused.as_str());
        let border_unfocused =
            alloc_color_from_hex(logger, conn, screen, config_file.border_unfocused.as_str());

        let gaps_inner = config_file.gaps_inner;
        let gaps_outer = config_file.gaps_outer;

        let mod_key = match config_file.mod_key.as_str() {
            "Mod1" => xcb::x::ModMask::N1, // alt
            "Mod2" => xcb::x::ModMask::N2, // num lock
            "Mod3" => xcb::x::ModMask::N3, // undefined
            "Mod4" => xcb::x::ModMask::N4, // meta (windows / command)
            "Mod5" => xcb::x::ModMask::N5, // some obscure key idk
            _ => {
                logger.log("invalid \"mod_key\" in config, using Mod4", LogLevel::Error);
                xcb::x::ModMask::N4
            }
        };

        let default_layout = match parse_layout(config_file.default_layout.clone()) {
            Ok(layout) => layout,
            Err(_) => {
                logger.log(
                    format!(
                        "invalid default layout \"{}\", using MasterStack",
                        config_file.default_layout
                    )
                    .as_str(),
                    LogLevel::Error,
                );
                Layout::MasterStack
            }
        };

        let parsed_keybinds = parse_keybinds(mod_key, config_file.keybinds);
        let keybinds = match parsed_keybinds {
            Ok(keybinds) => keybinds,
            Err(_) => {
                logger.log(
                    "failed to parse keybinds. using default minimal keybinds...",
                    LogLevel::Error,
                );

                // minimal keybind set literally only lets you quit
                // once reloading keybinds is added it will go here as well
                // TODO: show a small banner letting the user know that something went
                // wrong in keybind parsing
                vec![Keybind {
                    modifiers: mod_key | xcb::x::ModMask::SHIFT,
                    keysym: xkb::keysyms::KEY_q,
                    action: Action::Quit,
                }]
            }
        };

        let startup: Vec<String> = config_file.startup;

        Self {
            border_width,
            gaps_inner,
            gaps_outer,
            border_focused,
            border_unfocused,
            mod_key,
            default_layout,
            keybinds,
            startup,
        }
    }
}
