# taild

Pipe log output through `taild` to insert visual separators when the stream goes idle. Useful for distinguishing bursts of activity in long-running log streams.

## Install

```sh
cargo install --path .
```

## Usage

### Pipe from a command

```sh
some-command | taild
```

### Follow a file

```sh
taild -f app.log
```

## Options

| Flag | Default | Description |
|------|---------|-------------|
| `-t, --idle <SECONDS>` | `3` | Seconds of silence before an idle marker is printed |
| `-c, --color <COLOR>` | gray | Color of the idle marker (`#FF6600`, `red`, `brightblue`, …) |
| `-f, --follow <FILE>` | — | Follow a file instead of reading stdin |
| `-v, --verbose` | — | Print debug info to stderr |

### Named colors

`black`, `red`, `green`, `yellow`, `blue`, `magenta`, `cyan`, `white`,
and their `bright*` variants (`brightred`, `brightblue`, …).

Hex colors: `#RRGGBB` (e.g. `#FF6600`).

## How it works

- **Active**: lines are printed with timestamps and URLs highlighted.
- **Idle**: when no activity is detected for `--idle` seconds, a separator line is printed:
  ```
  [1]-[2024-01-15 12:34:56]----- idle -------
  ```
- **Resume**: when activity returns after idle, a blank line is inserted before the next log line to visually separate bursts.

## Examples

```sh
# Default 3-second idle threshold
kubectl logs -f my-pod | taild

# 10-second threshold, orange idle marker
kubectl logs -f my-pod | taild -t 10 -c '#FF6600'

# Follow a file
taild -f /var/log/nginx/access.log

# Follow a file with a 5-second threshold
taild -f app.log -t 5 -c red
```
