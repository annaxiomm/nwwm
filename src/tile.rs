use crate::{config::Config, err::NwwmError};
use std::collections::HashMap;

pub enum Layout {
    Columns,
    Monocle,
}

#[derive(Debug)]
pub struct LayoutParams {
    pub x: i32,
    pub y: i32,
    pub width: u32,
    pub height: u32,
}

pub fn monocle(
    scheight: u16,
    scwidth: u16,
    windows: Vec<xcb::x::Window>,
    config: &Config,
) -> Result<HashMap<xcb::x::Window, LayoutParams>, NwwmError> {
    let mut layoutmap = HashMap::new();

    let border_width = config.border_width;
    let window_width = scwidth as u32 - (2 * border_width);
    let window_height = scheight as u32 - (2 * border_width);

    for window in windows.into_iter() {
        layoutmap.insert(
            window,
            LayoutParams {
                x: 0,
                y: 0,
                height: window_height,
                width: window_width,
            },
        );
    }

    Ok(layoutmap)
}

pub fn columns(
    scheight: u16,
    scwidth: u16,
    windows: Vec<xcb::x::Window>,
    config: &Config,
) -> Result<HashMap<xcb::x::Window, LayoutParams>, NwwmError> {
    let mut layoutmap = HashMap::new();

    let window_count = windows.len();
    if window_count == 0 {
        return Ok(layoutmap);
    }
    let border_width = config.border_width;

    let available_width = scwidth as u32;
    let slot_width = available_width / window_count as u32;

    let mut x = 0;

    for (i, window) in windows.into_iter().enumerate() {
        let width = if i == window_count - 1 {
            available_width - x as u32
        } else {
            slot_width
        };

        // subtract borders here
        let client_width = width - 2 * border_width;

        layoutmap.insert(
            window,
            LayoutParams {
                x,
                y: 0,
                width: client_width,
                height: scheight as u32 - 2 * border_width,
            },
        );

        x += width as i32;
    }

    Ok(layoutmap)
}
