use crate::{config::Config, err::NwwmError, wm::Rect};
use std::collections::HashMap;

#[derive(Clone, Copy, Debug)]
#[allow(dead_code)]
pub enum Layout {
    Columns,
    Monocle,
    MasterStack,
}

pub fn monocle(
    screen: &Rect,
    windows: Vec<xcb::x::Window>,
    config: &Config,
) -> Result<HashMap<xcb::x::Window, Rect>, NwwmError> {
    let mut layoutmap = HashMap::new();

    let border_width = config.border_width;
    let window_width = screen.width - (2 * border_width);
    let window_height = screen.height - (2 * border_width);

    for window in windows.into_iter() {
        layoutmap.insert(
            window,
            Rect {
                x: 0,
                y: 0,
                width: window_width,
                height: window_height,
            },
        );
    }

    Ok(layoutmap)
}

pub fn columns(
    screen: &Rect,
    windows: Vec<xcb::x::Window>,
    config: &Config,
) -> Result<HashMap<xcb::x::Window, Rect>, NwwmError> {
    let mut layoutmap = HashMap::new();

    let window_count = windows.len();
    if window_count == 0 {
        return Ok(layoutmap);
    }
    let border_width = config.border_width;

    let available_width = screen.width;
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
            Rect {
                x,
                y: 0,
                width: client_width,
                height: screen.height - 2 * border_width,
            },
        );

        x += width as i32;
    }

    Ok(layoutmap)
}

pub fn master_stack(
    screen: &Rect,
    windows: Vec<xcb::x::Window>,
    config: &Config,
) -> Result<HashMap<xcb::x::Window, Rect>, NwwmError> {
    let mut layoutmap = HashMap::new();
    let window_count = windows.len();

    // if layout is empty
    if window_count == 0 {
        return Ok(layoutmap);
    }

    let screen_width = screen.width;
    let screen_height = screen.height;

    // if theres just a master - no stack
    if window_count == 1 {
        layoutmap.insert(
            windows[0],
            Rect {
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
        Rect {
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
            Rect {
                x: master_width as i32,
                y,
                width: stack_width - 2 * config.border_width,
                height: client_height,
            },
        );

        y += height as i32;
    }
    Ok(layoutmap)
}
