use crate::{config::Config, err::NwwmError};
use std::collections::HashMap;

#[derive(Clone, Copy, Debug)]
pub enum Layout {
    Columns,
    Monocle,
    MasterStack,
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

pub fn master_stack(
    scheight: u16,
    scwidth: u16,
    windows: Vec<xcb::x::Window>,
    config: &Config,
) -> Result<HashMap<xcb::x::Window, LayoutParams>, NwwmError> {
    let mut layoutmap = HashMap::new();
    let window_count = windows.len();

    // if layout is empty
    if window_count == 0 {
        return Ok(layoutmap);
    }

    let screen_width = scwidth as u32;
    let screen_height = scheight as u32;

    // if theres just a master - no stack
    if window_count == 1 {
        layoutmap.insert(
            windows[0],
            LayoutParams {
                x: 0,
                y: 0,
                width: screen_width - (2 * config.border_width),
                height: screen_height - (2 * config.border_width),
            },
        );
        return Ok(layoutmap);
    }

    // if there is a stack
    let master_width = screen_width / 2;
    let stack_width = screen_width - master_width;
    let stack_count = window_count as u32 - 1;

    layoutmap.insert(
        windows[0],
        LayoutParams {
            x: 0,
            y: 0,
            width: master_width - (2 * config.border_width),
            height: screen_height - (2 * config.border_width),
        },
    );

    let slot_height = screen_height / stack_count;
    let mut y = 0;

    for (i, window) in windows.into_iter().skip(1).enumerate() {
        let height = if i == stack_count as usize - 1 {
            screen_height - y as u32
        } else {
            slot_height
        };

        // subtract borders here
        let client_height = height - 2 * config.border_width;

        layoutmap.insert(
            window,
            LayoutParams {
                x: master_width as i32,
                y: y,
                width: stack_width - 2 * config.border_width,
                height: client_height,
            },
        );

        y += height as i32;
    }
    Ok(layoutmap)
}
