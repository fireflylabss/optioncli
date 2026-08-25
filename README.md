# ◆ opt

**opt** — the **Option** family CLI.

It is a thin dispatcher in the spirit of `git` / `cargo`: `opt <app> [args...]`
forwards to the matching app binary and mirrors its exit status. It also
offers family-level tools: `status`, `doctor`, `install`, `sys` and `version`.

```text
◆ opt — the Option family

  files     ◇   terminal file manager
  music     ♪   CLI music player
  notes     ◇   local-first markdown notes
  terminal  ◇   GTK4 terminal with tiling splits
  opsh      ◇   small local shell
  fat       ◇   fast syntax-aware cat
  needle    ⌕   instant local file search
```

## Install

Requires Rust 1.85+.

```bash
cargo install optioncli        # binary: `opt`
```

Or from the checkout:

```bash
cargo run --release -- status
```

## Usage

```bash
opt                        # list the family
opt files                  # run optionFiles interactively
opt music play song.mp3 -v 80
opt status                 # installed apps + versions
opt doctor                 # check each app's system dependencies
opt install                # install the whole family
opt install music          # install one app
opt update                 # update the whole family
opt sys clean              # remove each app's ~/.option/<id>/cache
opt sys info               # OS, arch, home and option root
opt sys path               # list PATH
opt version                # opt's own version
```

After the app id, **every** argument is forwarded untouched, so flags and
spaces reach the underlying binary as written:

```bash
opt files list "/tmp/com espaço"
opt music dl https://youtu.be/… --audio
```

### Aliases

`f` → `files`, `m` → `music`, `file` → `files`.

### Environment

| Variable | Meaning |
|----------|---------|
| `OPTION_BIN_<ID>` | Force an app's binary path (e.g. `OPTION_BIN_MUSIC`) |
| `OPTION_PKG` | `install`/`update` manager: `cargo` (default) or `yay` |
| `NO_COLOR` | Disable color in output (per no-color.org) |

### Routing & packages

| App    | Binary candidates | cargo | AUR |
|--------|-------------------|-------|-----|
| files  | `optionfiles`, `fls` | `optionfiles` | `optionfiles` |
| music  | `optionmusic`, `msc` | `optionmusic` | `optionmusic` |
| notes  | `nts` | `optionnotes` | `optionnotes` |
| terminal | `optionterm` | `optionterm` | `optionterm` |
| opsh   | `opsh` | `opsh` | `opsh` |
| fat    | `fat` | `ofat` | `ofat` |
| needle | `needle` | `needle` | `needle` |

The first candidate on `PATH` wins; `OPTION_BIN_<ID>` overrides it.

### `doctor`

For each app, `opt doctor` reports whether its binary is installed and checks
the system dependencies it needs (e.g. `libmpv`, `cava`, `yt-dlp` for music;
GTK4, libadwaita, VTE for the terminal; ImageMagick for file previews).

### Local-first

`opt` reads `~/.option` only for identity (marks, bundle ids via `optionSDK`);
it never creates files or services beyond cache cleanup. It is deliberately
small and local — no daemon, account, telemetry or cloud.

## Development

```bash
cargo fmt --check
cargo test
cargo build --release
```

## License

Apache License 2.0 — see [LICENSE](LICENSE).
