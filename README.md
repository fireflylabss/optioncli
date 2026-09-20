# ◆ opt

**opt** — the **Option** family CLI.

It is a thin dispatcher in the spirit of `git` / `cargo`: `opt <app> [args...]`
forwards to the matching app binary and mirrors its exit status. It also
offers family-level tools: `gui`, `status`, `doctor`, `install`, `sys` and
`version`.

```text
◆ opt — the Option family

  files     ◇   terminal file manager
  music     ♪   CLI music player
  cal       ◷   minimal local calendar
  terminal  ◇   GTK4 terminal with tiling splits
  opsh      ◇   small local shell
  fat       ◇   fast syntax-aware cat
  search    ⌕   instant local file search
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
opt gui files              # open the desktop file manager
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

`opt gui <app> [args...]` runs the app's desktop front-end instead of its
CLI (`gui` is a reserved verb, like `status`): files → `optionfiles-gtk`,
`fls-gtk`; music → `optionmusic-gpui`;
search → `optionsearch-gtk`, `needle`. Bare `opt gui`
lists them with their installed state; `OPTION_GUI_BIN_<ID>` overrides
the lookup.

### Aliases

`f` → `files`, `file` → `files`, `m` → `music`, `c` → `cal`,
`s` → `search`, `needle` → `search`, `nld` → `search`.

### Environment

| Variable | Meaning |
|----------|---------|
| `OPTION_BIN_<ID>` | Force an app's binary path (e.g. `OPTION_BIN_MUSIC`) |
| `OPTION_GUI_BIN_<ID>` | Force an app's GUI binary path (e.g. `OPTION_GUI_BIN_FILES`) |
| `OPTION_PKG` | `install`/`update` manager: `cargo` (default) \| `yay` \| `paru` \| `pacman` (`aur` = paru if present, else yay) |
| `NO_COLOR` | Disable color in output (per no-color.org) |

### Routing & packages

| App    | Binary candidates | GUI (`opt gui`) | cargo | AUR |
|--------|-------------------|-----------------|-------|-----|
| files  | `optionfiles`, `fls` | `optionfiles-gtk`, `fls-gtk` | `optionfiles` | `optionfiles` |
| music  | `optionmusic`, `msc` | `optionmusic-gpui` | `optionmusic` | `optionmusic` |
| cal    | `optioncalendar`, `oca` | — | `optioncalendar` | `optioncalendar` |
| terminal | `optionterm` | — | `optionterm` | `optionterm` |
| opsh   | `opsh` | — | `opsh` | `opsh` |
| fat    | `fat` | — | `ofat` | `ofat` |
| search | `optionsearch`, `nld`, `needle` | `optionsearch-gtk`, `needle` | `optionsearch-cli` | `optionsearch` |

The first candidate on `PATH` wins; `OPTION_BIN_<ID>` overrides it.
The same table is printed by `opt` (menu) and `opt help`.
`needle`/`nld` stay as compat aliases of the `needle` → `optionSearch`
rename (`opt needle` resolves to `search`).

### `install` / `update`

```bash
opt install            # whole family (cargo install <each>)
opt install music      # one app
opt install family     # AUR metapackage: yay -S option-family
opt update             # cargo install --force <each> | yay -Syu <family pkgs>
opt update music       # update one app
```

With `OPTION_PKG=yay|paru`, `install` runs `<helper> -S --noconfirm <aur>`
and `update` runs `<helper> -Syu --noconfirm <aur...>` (a real refresh
upgrade, not a reinstall). `OPTION_PKG=pacman` uses `sudo pacman -S/-Syu`.
`opt install family` installs the `option-family` metapackage
(`packaging/aur-option-family/PKGBUILD`: depends on `opt` + all apps)
in a single AUR transaction; under cargo it just installs every app.

### `doctor`

For each app, `opt doctor` reports whether its binary is installed and checks
the system dependencies it needs. Each dep is tagged `[req]` (required) or
`[opc]` (optional):

- files: `xdg-open` (req), `$EDITOR` — `$EDITOR`/`$VISUAL` or `vi`/`nano`
  fallback (req), `imagemagick` (`magick` or `convert`), clipboard
  (`wl-copy`/`xclip`/`xsel`/`pbcopy`), `gio`/`trash` (all optional).
- music: `mpv` binary + `libmpv` (`ldconfig -p`, `pkg-config --exists mpv`
  or the `mpv` binary; both req), `cava`, `yt-dlp`, `ffmpeg` (optional).
- terminal: `gtk4`, `libadwaita`, `vte-2.91-gtk4` via
  `pkg-config --exists` with a `pacman -Q` fallback (req), plus optional
  `pkg-config` probe note.
- needle: `pdftotext` (optional; sqlite is embedded, no check needed).
- cal, opsh, fat: no extra system deps.

Missing required deps show as `faltando (req)`, missing optional ones as
`ausente (opc)`, each with a pacman hint plus the apt/dnf equivalents.
The report is visual only — it never changes the exit status.

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
