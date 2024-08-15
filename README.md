# ttysvr

Screensavers for your terminal.

Uses [bevy_ratatui_render](https://github.com/cxreiff/bevy_ratatui_render), my
bevy plugin that allows you to render a bevy application to the terminal using
[ratatui](https://github.com/ratatui-org/ratatui) and
[ratatui-image](https://github.com/benjajaja/ratatui-image).

Triggering immediately works in any shell, triggering after a delay is currently Zsh only.

## installation

```sh
cargo install --locked ttysvr`
```

## usage

Starts the screensaver immediately. If no variant is specified, one is randomly selected.
```sh
ttysvr [VARIANT]
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

|         |                     |
|---------|---------------------|
| maze    | not yet implemented |
| pipes   | not yet implemented |
| bubbles | not yet implemented |

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

