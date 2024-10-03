# ttysvr

Screensavers for your terminal. Start immediately or after a period of inactivity within a shell.

<p float="left">
<img src="https://assets.cxreiff.com/github/ttysvr_logo.gif" width="33%">
<img src="https://assets.cxreiff.com/github/ttysvr_bubbles.gif" width="33%">
<img src="https://assets.cxreiff.com/github/ttysvr_maze.gif?" width="33%">
<p>

Uses [bevy_ratatui_render](https://github.com/cxreiff/bevy_ratatui_render), my
bevy plugin that allows you to render a bevy application to the terminal using
[ratatui](https://github.com/ratatui-org/ratatui) and
[ratatui-image](https://github.com/benjajaja/ratatui-image).

Triggering the screensaver immediately works in any shell, triggering after a set period of inactivity is currently Zsh only.

## installation

```sh
# cargo
cargo install --locked ttysvr
```
```sh
# homebrew
brew install cxreiff/tap/ttysvr
```

## usage

Starts the screensaver immediately. If no variant is specified, one is randomly selected.
```sh
ttysvr [VARIANT]
```

Some variants have subvariants.

```sh
ttysvr [VARIANT] [SUBVARIANT]
```

Sets up the screensaver to activate after `DELAY` seconds of inactivity in your current shell session.
```sh
eval `ttysvr [VARIANT] --init [DELAY]`
```

Cancels the screensaver in your current shell session.
```sh
eval `ttysvr --cancel`
```

> [!IMPORTANT]
> Note that the `--init` and `--cancel` options require being wrapped in `eval` and backticks.
> This is because activating the screensaver after a set period of inactivity relies on setting
> the TMOUT environment variable in your _current shell_, instead of the child shell created by
> the `ttysvr` process. Issues/PRs are greatly appreciated if somebody is aware of a better way
> of doing this!

## variants

| variant | subvariants? | description                                |
|---------|--------------|--------------------------------------------|
| bubbles |              | Colorful bubbles bounce around the screen. |
| logo    | `dvd`, `tty` | DVD player style bouncing logo.            |
| maze    |              | 3D randomly generated maze.                |

## compatibility

### zsh only

The screensaver delay works based on setting a session timeout and catching the
ALRM signal in a way only compatible with ZSH. If you know a way to achieve something
similar in another shell, please open an issue!

### terminal

This requires that your terminal:

1. Supports 24bit color.
2. Has reasonably efficient rendering.

This includes a decent variety of terminals, but I have personally confirmed good results in the following:

- Alacritty (macOS, linux)
- Kitty (macOS)
- WezTerm (macOS)
- iTerm2 (macOS)

