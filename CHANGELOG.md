# Changelog

We follow [Semantic Versioning](https://semver.org/) and [Keep a Changelog](https://keepachangelog.com/). `optioncli` is a single CLI surface.

<details>
<summary>To see more about versioning, expand this.</summary>

Every version string starts with `v` (required), e.g. `v0.1.0-stable`, `v0.1.0`.

Here the installable surface is **CLI** (`opt`).

With one surface there is no `m` in the tag and no per-surface sections — just the version notes.

Each release heading is the version and date; under it, a short summary ends with a plain sentence naming the surface and tag.

</details>

## v0.1.1-stable · 04/09/2026

Refines doctor checks, help/menu routing, and install/update flows on the single `opt` surface. This version was made for CLI with a stable release channel on 04/09/2026 (v0.1.1-stable).

- `opt doctor` tags every system dep as required or optional: `xdg-open` (req), `$EDITOR` (req), and `libmpv` (req, via `ldconfig`, `pkg-config`, or the `mpv` binary).
- `opt doctor` checks GTK4, libadwaita, and VTE (req, via `pkg-config` with a `pacman -Q` fallback), plus optional `magick`|`convert`, clipboard providers, and `gio`/`trash`.
- Optional media/docs deps stay visual-only: `cava`, `yt-dlp`, `ffmpeg`, and `pdftotext` report as `ausente (opc)` without changing the exit status.
- `opt` menu and `opt help` print the same routing table (app | bins | cargo | AUR | about) with aliases `f`/`file` → files, `m` → music, `c` → cal.
- `opt install` and `opt update` are now separate flows with `OPTION_PKG` supporting `cargo` | `yay` | `paru` | `pacman`, and `opt install family` installs the `option-family` metapackage in one AUR transaction.
- New `option-family` metapackage (`packaging/aur-option-family/PKGBUILD`) depends on `opt` plus every Option app.
- Various other small tweaks.

## v0.1.0-stable · 24/08/2026

Initial Option-family CLI. This version was made for CLI with a stable release channel on 24/08/2026 (v0.1.0-stable).

- `opt <app> [args...]` forwards to the matching app binary (files, music, notes, terminal, opsh, fat, needle) with untouched arguments.
- `opt` (no args) prints the family menu; `opt help` prints usage; `opt version` prints the opt version.
- `opt status` reports each installed app and its version, detecting binaries on `PATH` and skipping GUI apps that launch on `--version`.
- `opt doctor` checks each app's binary and its system dependencies (libmpv, cava, yt-dlp, GTK4, libadwaita, VTE, ImageMagick, pdftotext, $EDITOR).
- `opt install` / `opt update` install or update the family (or specific apps) via `cargo install`, or `yay -S` with `OPTION_PKG=yay`.
- `opt sys clean|info|path` — remove app caches, print system info, list `PATH`.
- Binary resolution searches `PATH` in order and supports `OPTION_BIN_<ID>` overrides.
- Forwarded subprocesses inherit stdio and mirror the child's exit status, including signal-terminated children.
- Friendly errors for unknown apps and missing binaries, with install hints.
- Alias support: `f` → files, `m` → music.
