<div align="center">
<h1>nwwm</h1>
<p>an experimental window manager for x11, written in rust</p>
<img alt="GitHub Actions Workflow Status" src="https://img.shields.io/github/actions/workflow/status/annaxiomm/nwwm/rust.yml">
<img alt="GitHub License" src="https://img.shields.io/github/license/annaxiomm/nwwm" />
<br/>
<sub><a href="https://github.com/annaxiomm/nwwm">GitHub</a> ⋅ <a href="https://codeberg.org/annaxiomm/nwwm">Codeberg (mirror)</a></sub>
<br/><br/>
</div>

![a screenshot of nwwm](./screenshot.png)

> [!WARNING]
> nwwm is a learning project and isn't intended to be a production-ready window manager. use at your own risk!

## Features
- dynamic tiling
- TOML configuration
- workspaces
- dock awareness
- EWMH support
- X11 native

## Why build a window manager in {year}?
**Because I can.** nwwm isn't meant to be a production ready, top of the line, solves all your problems window manager. I built nwwm because I wanted to learn how this kind of stuff works.

## Building

> nwwm is an experimental project and isn't currently available through the AUR or other package managers.

### You will need:
- Linux
- Rust and Cargo
- X11
- Xephyr
- Some form of git

### Installation and running
This guide walks you through running nwwm inside of Xephyr, a nested X server that runs as a window inside of your existing session. This **WILL NOT** replace your current window manager.
1. Clone the repository using your method of choice
    1. **With Git**: `git clone https://github.com/annaxiomm/nwwm`
    2. With Github CLI: `gh repo clone annaxiomm/nwwm`
2. Enter the directory with `cd nwwm`
3. Run the `./test.sh` script. This will open a Xephyr window with nwwm running
4. Focus the Xephyr window and press `ctrl+shift` on your keyboard to give nwwm keyboard focus
5. Go ham!

## Configuration
nwwm is configured via a TOML config file. This is, by default, located at `~/.config/nwwm/config.toml` (or wherever your config directory is). If no configuration file is present, nwwm will attempt to create one using its defaults.

### Options
nwwm can be configured using the following options:
| Option | Description | Default |
| ------------- | -------------- | -------------- |
| `border_focused` | the colour of a window's border when focused  | `#ffa500` |
| `border_unfocused` | the colour of a window's border when unfocused | `#ffffff` |
| `border_width` | the width of a window's border in pixels | `2` |
| `mod_key` | the modifier key to be used in keybindings | `Mod4` (Windows / Command key) |
| `gaps_outer` | the width of the outer gaps (between the windows and the edge of the screen) in pixels | `10` |
| `gaps_inner` | the width of the inner gaps (between each window) in pixels | `10` |
| `default_layout` | the default tiling layout to be used in each workspace | `masterstack` |

`mod_key` can be set to the following values:
- `Mod1` - alt
- `Mod2` - number lock
- `Mod3` - nothing (don't use this one)
- `Mod4` - meta key (windows / command depending on the keyboard)
- `Mod5` - alt gr

`default_layout` can be set to the following values:
- `masterstack` - one window takes up half the screen (the *master*) and the others are tiled vertically in the other half (the *stack*)
- `columns` - each window takes up an equal amount of space, divided horizontally
- `monocle` - each window takes up the whole screen, and the currently focused window is brought to the front

### Keybindings
Keybindings in nwwm are configured using the following syntax:
```toml
keybinds = {
  "modifiers+key" = "action"
}
```
for example: 

```toml
"mod+return" = "exec kitty",
"mod+shift+q" = "quit"
```

#### Modifiers
- `mod` - whatever is set as `mod_key`
- `shift`
- `alt`
- `ctrl` - control

#### Actions
| Action | Description | Values |
| ------------- | -------------- | -------------- |
| `exec <command>` | runs `<command>` | `<command>` can be any string that represents a valid shell command on your system (e.g. `kitty`, `rofi -show run`) |
| `closewindow` | closes the currently focused window | |
| `setworkspace <workspace>` | sets the current workspace to `<workspace>` | `<workspace>` can be any positive integer (whole number) greater than 0 |
| `focus <direction>` | moves window focus | `<direction>` can either be `next` or `last` |
| `quit` | quits nwwm | |
| `setlayout <layout>` | sets the layout of the current workspace | `<layout>` can be `monocle`, `masterstack`, or `columns` |
a full example config can be found at [config/default.toml](config/default.toml)

## What I learned
- X11 is an ancient beast
- (good) Error handling is actually a lot harder than I thought
- code that runs != good code
- I'm actually really bad at rust
- How to structure a low-level project properly
- Knowing my limits
- Defining a clear end point and working towards it

## Known limitations
nwwm is a small experimental project and isn't meant to replace any existing window managers. Rather, it was created to help me become a better programmer and learn my faults before I try to work on something actually important.

- **X11 only** - nwwm is built on XCB and does not support Wayland.
- **Limited EWMH support** - nwwm only implements a small subset of the EWMH (Extended Window Manager Hints) specification and as such some applications may behave unexpectedly
- **Dock support is basic** - nwwm understands docks and their reserved screen space but this hasn't been extensively tested
- **No persistent state** - nwwm currently does not recognise previously opened windows, does not save state between sessions, and unexpected termination of nwwm may lead to data loss. 
- **Tiling** - ratios cannot be changed, and tiling is done algorithmically on a Vector of windows so more advanced layouts like BSP cannot be easily implemented
- **Error handling** - nwwm's error handling is very naive and many errors may go unnoticed or cause nwwm to quit without warning instead of attempting to recover
- **Instability** - nwwm is experimental software and is as such very unstable
- **Limited configurability** - nwwm's configuration is intentionally minimal and doesn't expose every aspect of the window manager
- **No multi-monitor support** - nwwm treats the entire X11 screen as a single monitor and does not support splitting workspaces across monitors
- **Testing is limited** - nwwm has been primarily tested within Xephyr on my own X11 setup

These limitations are intentional to some extent - nwwm was developed to help me learn about X11, Linux desktop interactions, and writing low level software rather than to compete with existing window managers.

## Future ideas
In the future, I will develop a successor to nwwm using what I've learned to make an actually viable and useful window manager, preferably implementing some of the following ideas:

- Lua scripting for configuration
- Multi-monitor Xinerama support
- Actually half decent error handling that attempts to recover and tells you what went wrong
- More advanced tiling, like BSP or full floating
- Built in dock and cursor
- Hot config reloading
- Potentially compositing behaviour, such as animations or transparency
- Full EWMH compatibility
- Most importantly, daily driver potential

## Credits
Finally, a list of people who helped me throughout the development process

- [Cooki](https://github.com/cooki-studios) for his continued reassurance that nwwm is, indeed, tuff
- [Hack Club](https://hackclub.com) for encouraging me to start this project
