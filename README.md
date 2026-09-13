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
> nwwm is a learning project and isn't intended to be a production-ready window manager. use at your own risk.

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

`mod_key` can be set to the following values:
- `Mod1` - alt
- `Mod2` - number lock
- `Mod3` - nothing (don't use this one)
- `Mod4` - meta key (windows / command depending on the keyboard)
- `Mod5` - alt gr
```

### Keybindings

Keybindings in nwwm are configured using the following syntax:



