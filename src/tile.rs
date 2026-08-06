use crate::err::NwwmError;
use std::collections::HashMap;

pub enum Layout {
    BasicTile,
}

#[derive(Debug)]
pub struct LayoutParams {
    pub x: i32,
    pub y: i32,
    pub width: u32,
    pub height: u32,
}

pub fn basic(
    scheight: u16,
    scwidth: u16,
    windows: Vec<xcb::x::Window>,
) -> Result<HashMap<xcb::x::Window, LayoutParams>, NwwmError> {
    let mut layoutmap = HashMap::new();

    let window_count = windows.len();
    if window_count == 0 {
        return Ok(layoutmap);
    }
    let window_width: u32 = (scwidth / window_count as u16) as u32;

    let mut start: i32 = 0;

    windows.into_iter().for_each(|w| {
        layoutmap.insert(
            w,
            LayoutParams {
                x: start,
                y: 0,
                width: window_width,
                height: scheight as u32,
            },
        );

        start += window_width as i32;
    });

    Ok(layoutmap)
}
