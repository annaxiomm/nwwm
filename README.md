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
- [x] ewmh compilance (extremely limited)
- [ ] more stuff coming soon!

## Installation / testing
**you will need:**
- rust & cargo
- xorg-server and xorg-xinit
- Xephyr

I severely recommend that you DO NOT install nwwm as it is. it's 100% unusable and you will be stuck with a black screen (or whatever your wallpaper is).

to test it, make sure you aren't using display 2 for anything then run `./test.sh &`. this will launch a Xephyr instance with nwwm running, and it will also launch kitty within nwwm (which you can then use to open other apps).

If you want to make changes to the test script feel free, but **for the love of god do not commit them**
