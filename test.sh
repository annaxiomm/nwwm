#!/usr/bin/bash
Xephyr :2 -screen 1280x720 &
DISPLAY=:2 cargo run &
