#!/usr/bin/bash
Xephyr :2 &
DISPLAY=:2 cargo run &
DISPLAY=:2 kitty &
