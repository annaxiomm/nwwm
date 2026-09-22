// TILE.rs
// -------
// tiling logic

use crate::{config::Config, err::NwwmError, wm::Rect};
use std::collections::HashMap;

#[derive(Clone, Copy, Debug)]
pub enum Layout {
    Columns,
    Monocle,
    MasterStack,
}

// every window takes up 100% of the screen,
// and the currently focused window is brought
// to the top
pub fn monocle(
    screen: &Rect,
    windows: Vec<xcb::x::Window>,
    config: &Config,
) -> Result<HashMap<xcb::x::Window, Rect>, NwwmError> {
    let mut layoutmap = HashMap::new();

    let border_width = config.border_width;
    let window_width = screen.width - (2 * border_width) - (2 * config.gaps_outer);
    let window_height = screen.height - (2 * border_width) - (2 * config.gaps_outer);

    for window in windows.into_iter() {
        layoutmap.insert(
            window,
            Rect {
                x: screen.x + config.gaps_outer as i32,
                y: screen.y + config.gaps_outer as i32,
                width: window_width,
                height: window_height,
            },
        );
    }

    Ok(layoutmap)
}

// columns can probably be removed, it's 100% obsolete
// but i'm keeping it anyways
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
    let outer_gap = config.gaps_outer;
    let inner_gap = config.gaps_inner;

    let total_inner_gaps = inner_gap * (window_count - 1) as u32;
    let available_width = screen.width - (2 * outer_gap) - total_inner_gaps;
    let slot_width = available_width / window_count as u32;

    let mut x = screen.x + outer_gap as i32;
    let right_edge = screen.x + screen.width as i32 - outer_gap as i32;

    for (i, window) in windows.into_iter().enumerate() {
        let width = if i == window_count - 1 {
            (right_edge - x) as u32
        } else {
            slot_width
        };

        let client_width = width - (2 * border_width);

        layoutmap.insert(
            window,
            Rect {
                x,
                y: screen.y + outer_gap as i32,
                width: client_width,
                height: screen.height - (2 * border_width) - (2 * outer_gap),
            },
        );

        x += width as i32;
        if i < window_count - 1 {
            x += inner_gap as i32;
        }
    }

    Ok(layoutmap)
}

// 1 window is the master which takes up half the screen
// the rest form a "stack" and tile vertically on the
// other half
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

    let outer_gap = config.gaps_outer;
    let inner_gap = config.gaps_inner;

    let screen_width = screen.width - (2 * outer_gap);
    let screen_height = screen.height - (2 * outer_gap);

    // if theres just a master - no stack
    if window_count == 1 {
        layoutmap.insert(
            windows[0],
            Rect {
                x: screen.x + outer_gap as i32,
                y: screen.y + outer_gap as i32,
                width: screen_width - (2 * config.border_width),
                height: screen_height - (2 * config.border_width),
            },
        );
        return Ok(layoutmap);
    }

    let available_width = screen_width - inner_gap;

    // if there is a stack
    let master_width = available_width / 2;
    let stack_width = available_width - master_width;
    let stack_count = window_count as u32 - 1;
    let stack_height = screen_height - inner_gap * (stack_count - 1);

    layoutmap.insert(
        windows[0],
        Rect {
            x: screen.x + outer_gap as i32,
            y: screen.y + outer_gap as i32,
            width: master_width - (2 * config.border_width),
            height: screen_height - (2 * config.border_width),
        },
    );

    let slot_height = stack_height / stack_count;
    let mut y = 0;

    for (i, window) in windows.into_iter().skip(1).enumerate() {
        let height = if i == stack_count as usize - 1 {
            stack_height - y as u32
        } else {
            slot_height
        };

        // subtract borders here
        let client_height = slot_height - 2 * config.border_width;

        layoutmap.insert(
            window,
            Rect {
                x: master_width as i32 + screen.x + (outer_gap + inner_gap) as i32,
                y: y + screen.y + outer_gap as i32,
                width: stack_width - 2 * config.border_width,
                height: client_height,
            },
        );

        y += height as i32;

        if i < window_count - 1 {
            y += inner_gap as i32;
        }
    }
    Ok(layoutmap)
}
