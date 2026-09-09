<div align="center">
<h1>nwwm</h1>
<p>an experimental window manager for x11</p>
<img alt="GitHub Actions Workflow Status" src="https://img.shields.io/github/actions/workflow/status/annaxiomm/nwwm/rust.yml">
<img alt="GitHub License" src="https://img.shields.io/github/license/annaxiomm/nwwm" />
<br/>
<sub><a href="https://github.com/annaxiomm/nwwm">GitHub</a> ⋅ <a href="https://codeberg.org/annaxiomm/nwwm">Codeberg (mirror)</a></sub>
<br/><br/>
</div>

![a screenshot of nwwm](./screenshot.png)

## Features
- [x] multiple tiling modes
  - [x] columns
  - [x] monocle
  - [x] master/stack
  - [ ] dwindle
- [x] keybinds
  - [x] modular?
  - [ ] customisable?
- [x] cool cosmetic stuff
  - [x] window borders
  - [ ] vanity gaps
- [x] multiple desktops
  - [x] switching?
  - [ ] moving windows?
  - [x] bar compatibility?
- [x] ewmh compilance (extremely limited)
- [ ] more stuff coming soon!

## Installation / testing
**you will need:**
- rust & cargo
- xorg-server and xorg-xinit
- Xephyr

You should not install nwwm as your main window manager. It is experimental software and lacks many basic features you would expect from a window manager. **You have been warned.**

To test nwwm, make sure you aren't using display 2 for anything then run `./test.sh &`. This will launch a Xephyr instance with nwwm running. Then, click on the window, press `Ctrl+Shift` to let Xephyr grab your keyboard, then go ham!

If you want to make changes to the test script feel free, but **for the love of god do not commit them**

## EWMH Checklist
### WM
- [x] _NET_WM_NAME
- [x] _NET_SUPPORTING_WM_CHECK
- [x] _NET_SUPPORTED
- [x] _NET_CLIENT_LIST
### Desktops
- [x] _NET_NUMBER_OF_DESKTOPS
- [ ] _NET_DESKTOP_GEOMETRY
- [ ] _NET_DESKTOP_VIEWPORT
- [x] _NET_CURRENT_DESKTOP
- [ ] _NET_DESKTOP_NAMES
### etc
- [ ] _NET_ACTIVE_WINDOW
- [ ] _NET_WORKAREA
