//! Known Option family apps, their binary names, aliases and lookup helpers.
//!
//! `opt <app> [args...]` resolves `<app>` to a concrete binary on `PATH` and
//! forwards the remaining arguments to it. Individual apps stay independent;
//! this module also knows install/doctor metadata for each.

use std::path::PathBuf;
use std::process::Command;

/// A system dependency that an Option app may need.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SysDep {
    /// Detection key (see [`dep_present`]). Not always a binary name:
    /// e.g. `editor`, `clipboard`, `imagemagick`, `libmpv`, `gtk4`.
    pub name: &'static str,
    /// Short human label shown in `doctor`.
    pub label: &'static str,
    /// One-line why / how to install (pacman-first).
    pub hint: &'static str,
    /// True when the app cannot do its core job without this dep.
    /// Missing required deps are reported as `faltando (req)` by `doctor`;
    /// missing optional ones as `ausente (opc)`.
    pub required: bool,
}

/// The resolved identity of one family app.
#[derive(Debug, Clone, Copy)]
pub struct AppSpec {
    /// Canonical id used on the command line, e.g. `music`.
    pub id: &'static str,
    /// One-char mark used by optionSDK (◆ ❯ ♪ ◷ ◇ ⌕).
    pub mark: &'static str,
    /// Binary names to try, in order. The first one present on `PATH` wins.
    pub bins: &'static [&'static str],
    /// Optional desktop front-end binaries, tried in order by `opt gui`.
    /// Empty when the app has no separate GUI bin — `terminal`'s only
    /// bin is already the GUI (see `is_gui`).
    pub gui_bins: &'static [&'static str],
    /// What the app does, for the menu / help.
    pub about: &'static str,
    /// Cargo package name (`cargo install <cargo>`).
    pub cargo: &'static str,
    /// AUR package name (`yay -S <aur>`).
    pub aur: &'static str,
    /// Cargo package shipping the `gui_bins`, when it differs from `cargo`
    /// (e.g. `optionsearch-gui` vs the CLI's `optionsearch-cli`).
    /// Empty when the app has no separate GUI package.
    pub gui_cargo: &'static str,
    /// AUR package shipping the `gui_bins`. Empty when no AUR package
    /// installs them — `optionfiles`/`optionmusic` currently package only
    /// their CLI binaries, so cargo is the only route to their front-ends.
    pub gui_aur: &'static str,
    /// System dependencies checked by `opt doctor`.
    pub deps: &'static [SysDep],
    /// True for desktop/GUI apps whose `--version` launches the app instead
    /// of printing a version (e.g. optionTerm). `status` skips running them.
    pub is_gui: bool,
}

/// Short aliases accepted on the command line, mapping to a canonical id.
const ALIASES: &[(&str, &str)] = &[
    ("f", "files"),
    ("file", "files"),
    ("m", "music"),
    ("c", "cal"),
    ("s", "search"),
    ("needle", "search"),
    ("nld", "search"),
];

/// All accepted aliases (alias → canonical id), for help text.
pub fn aliases() -> &'static [(&'static str, &'static str)] {
    ALIASES
}

/// Read `<prefix>_<ID>` as an override path, trimmed.
///
/// Surrounding whitespace is stripped so a value pasted with a trailing
/// space still resolves; an all-whitespace value is treated as unset.
fn path_override(prefix: &str, id: &str) -> Option<PathBuf> {
    let raw = std::env::var(format!("{prefix}_{}", id.to_ascii_uppercase())).ok()?;
    let trimmed = raw.trim();
    (!trimmed.is_empty()).then(|| PathBuf::from(trimmed))
}

/// `OPTION_BIN_<ID>` environment override for a binary path.
fn bin_override(id: &str) -> Option<PathBuf> {
    // OPTION_BIN_TERMINAL → "terminal"
    path_override("OPTION_BIN", id)
}

/// `OPTION_GUI_BIN_<ID>` environment override for a GUI binary path.
fn gui_bin_override(id: &str) -> Option<PathBuf> {
    // OPTION_GUI_BIN_MUSIC → "optionmusic-gpui"
    path_override("OPTION_GUI_BIN", id)
}

/// Look up an app by its canonical id or a short alias.
pub fn lookup(id: &str) -> Option<&'static AppSpec> {
    let canonical = ALIASES
        .iter()
        .find(|(alias, _)| *alias == id)
        .map(|(_, target)| *target)
        .unwrap_or(id);
    APPS.iter().find(|a| a.id == canonical)
}

/// All canonical apps, in status / menu order (aliases excluded).
pub fn all() -> &'static [AppSpec] {
    APPS
}

/// Find a concrete binary path for an app, respecting `OPTION_BIN_*`.
///
/// Returns `None` when no candidate binary is on `PATH`.
pub fn find_binary(spec: &AppSpec) -> Option<PathBuf> {
    if let Some(path) = bin_override(spec.id) {
        return Some(path);
    }
    spec.bins.iter().find_map(|bin| which(bin))
}

/// Find a concrete desktop front-end binary for an app, respecting
/// `OPTION_GUI_BIN_*`.
///
/// Returns `None` when the app has no `gui_bins` or none is on `PATH`.
pub fn find_gui_binary(spec: &AppSpec) -> Option<PathBuf> {
    if let Some(path) = gui_bin_override(spec.id) {
        return Some(path);
    }
    spec.gui_bins.iter().find_map(|bin| which(bin))
}

/// True when the app is already its own desktop surface (`is_gui` with no
/// separate front-end, e.g. optionTerm), so `opt gui <app>` runs `bins`.
fn is_own_desktop_surface(spec: &AppSpec) -> bool {
    spec.is_gui && spec.gui_bins.is_empty()
}

/// True when `opt gui <app>` has anything to run at all.
pub fn has_desktop_surface(spec: &AppSpec) -> bool {
    !spec.gui_bins.is_empty() || spec.is_gui
}

/// The binaries `opt gui <app>` would try, in order — its own `bins` when
/// the app is already the desktop surface.
pub fn desktop_bins(spec: &AppSpec) -> &'static [&'static str] {
    if is_own_desktop_surface(spec) {
        spec.bins
    } else {
        spec.gui_bins
    }
}

/// Resolve the binary `opt gui <app>` should run.
pub fn find_desktop_binary(spec: &AppSpec) -> Option<PathBuf> {
    if is_own_desktop_surface(spec) {
        find_binary(spec)
    } else {
        find_gui_binary(spec)
    }
}

/// Search `PATH` for a single executable name, returning its path when found.
///
/// A file only counts when it is actually executable, so a non-executable
/// leftover on `PATH` is never reported as an installed app.
pub fn which(name: &str) -> Option<PathBuf> {
    let path = std::env::var_os("PATH")?;
    for dir in std::env::split_paths(&path) {
        let candidate = dir.join(name);
        if is_executable(&candidate) {
            return Some(candidate);
        }
    }
    None
}

/// True when `path` is a regular file the current user can execute.
///
/// On Unix this is an execute bit; elsewhere it falls back to a runnable
/// extension, so a stray `README.txt` on `PATH` is never mistaken for a
/// binary. Option apps target Unix — the other arm only keeps a
/// cross-compile honest.
fn is_executable(path: &std::path::Path) -> bool {
    let Ok(metadata) = std::fs::metadata(path) else {
        return false;
    };
    if !metadata.is_file() {
        return false;
    }
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        metadata.permissions().mode() & 0o111 != 0
    }
    #[cfg(not(unix))]
    {
        let ext = path
            .extension()
            .and_then(|ext| ext.to_str())
            .map(str::to_ascii_lowercase);
        matches!(ext.as_deref(), Some("exe" | "cmd" | "bat" | "com"))
    }
}

/// True when `name` is present as an executable on `PATH`.
pub fn has_binary(name: &str) -> bool {
    which(name).is_some()
}

/// True when the system dependency `dep` is satisfied.
///
/// Some deps are not single binaries and need custom detection:
/// - `editor`: `$EDITOR` (or `$VISUAL`) pointing at something installed,
///   else a `vi`/`nano` fallback.
/// - `clipboard`: any provider (`wl-copy`, `xclip`, `xsel`, `pbcopy`).
/// - `imagemagick`: `magick` (v7) or `convert` (v6).
/// - `libmpv`: `mpv` binary, `pkg-config --exists mpv`, `ldconfig -p`
///   showing `libmpv`, or a well-known `.so` path.
/// - `gtk4` / `libadwaita` / `vte-2.91`: `pkg-config --exists <module>`,
///   falling back to `pacman -Q <pkg>`.
/// - `gio`: `gio` or `trash` binary.
///
/// Everything else falls back to [`has_binary`] on `dep.name`.
pub fn dep_present(dep: &SysDep) -> bool {
    match dep.name {
        "editor" => editor_present(),
        "clipboard" => ["wl-copy", "xclip", "xsel", "pbcopy"]
            .iter()
            .any(|bin| has_binary(bin)),
        "imagemagick" => has_binary("magick") || has_binary("convert"),
        "libmpv" => libmpv_present(),
        "gtk4" => pkg_config_exists("gtk4") || pacman_owns("gtk4"),
        "libadwaita" => pkg_config_exists("libadwaita-1") || pacman_owns("libadwaita"),
        "vte-2.91" | "vte-2.91-gtk4" => pkg_config_exists("vte-2.91-gtk4") || pacman_owns("vte4"),
        "gio" => has_binary("gio") || has_binary("trash"),
        "pkg-config" => has_binary("pkg-config"),
        _ => has_binary(dep.name),
    }
}

/// apt/dnf package names mirroring the pacman-first [`SysDep::hint`],
/// shown by `doctor` when a dep is missing.
pub fn alt_hint(name: &str) -> (&'static str, &'static str) {
    match name {
        "mpv" | "libmpv" => ("mpv / libmpv2", "mpv"),
        "cava" => ("cava", "cava"),
        "yt-dlp" => ("yt-dlp", "yt-dlp"),
        "ffmpeg" => ("ffmpeg", "ffmpeg"),
        "imagemagick" => ("imagemagick", "ImageMagick"),
        "xdg-open" => ("xdg-utils", "xdg-utils"),
        "clipboard" => ("wl-clipboard | xclip", "wl-clipboard | xclip"),
        "gio" => ("libglib2.0-bin", "glib2"),
        "editor" => ("nano (ou $EDITOR)", "nano (ou $EDITOR)"),
        "gtk4" => ("libgtk-4-1", "gtk4"),
        "libadwaita" => ("libadwaita-1-0", "libadwaita"),
        "vte-2.91" | "vte-2.91-gtk4" => ("libvte-2.91-gtk4-0", "vte291-gtk4"),
        "pkg-config" => ("pkg-config", "pkgconf-pkg-config"),
        "pdftotext" => ("poppler-utils", "poppler-utils"),
        _ => ("—", "—"),
    }
}

fn editor_present() -> bool {
    for var in ["EDITOR", "VISUAL"] {
        if let Ok(value) = std::env::var(var) {
            let first = value.split_whitespace().next().unwrap_or("");
            if first.is_empty() {
                continue;
            }
            // A bare name is resolved through PATH; anything with a path
            // component (`./bin/ed`, `/usr/bin/vi`) is checked in place.
            // Either way the file has to be runnable, not merely present.
            let path = std::path::Path::new(first);
            if path.components().count() > 1 || path.is_absolute() {
                if is_executable(path) {
                    return true;
                }
            } else if has_binary(first) {
                return true;
            }
            // Set but not runnable: fall through to the vi/nano fallback.
        }
    }
    has_binary("vi") || has_binary("nano")
}

fn libmpv_present() -> bool {
    if has_binary("mpv") {
        return true;
    }
    if pkg_config_exists("mpv") {
        return true;
    }
    if let Ok(output) = Command::new("ldconfig").arg("-p").output() {
        if output.status.success() {
            let text = String::from_utf8_lossy(&output.stdout);
            if text.lines().any(|line| line.contains("libmpv")) {
                return true;
            }
        }
    }
    [
        "/usr/lib/libmpv.so",
        "/usr/lib/libmpv.so.2",
        "/usr/local/lib/libmpv.so",
    ]
    .iter()
    .any(|p| std::path::Path::new(p).exists())
}

fn pkg_config_exists(module: &str) -> bool {
    if !has_binary("pkg-config") {
        return false;
    }
    Command::new("pkg-config")
        .args(["--exists", module])
        .status()
        .map(|status| status.success())
        .unwrap_or(false)
}

fn pacman_owns(pkg: &str) -> bool {
    Command::new("pacman")
        .args(["-Q", pkg])
        .output()
        .map(|output| output.status.success())
        .unwrap_or(false)
}

/// Run `<bin> --version`, returning the first non-empty output line.
///
/// Some binaries print the version on stdout (e.g. `fls 0.2.5`); others use a
/// banner. Capture stdout and return its first trimmed line.
pub fn binary_version(bin: &std::path::Path) -> Option<String> {
    let output = Command::new(bin).arg("--version").output().ok()?;
    if !output.status.success() {
        return None;
    }
    let text = String::from_utf8_lossy(&output.stdout);
    text.lines()
        .map(str::trim)
        .find(|line| !line.is_empty())
        .map(str::to_owned)
}

/// Installation hint for a missing binary.
pub fn install_hint(spec: &AppSpec) -> String {
    format!("cargo install {}   (ou: yay -S {})", spec.cargo, spec.aur)
}

/// Installation hint for a missing desktop front-end.
///
/// The front-end often lives in its own package (`optionsearch-gui` vs the
/// CLI's `optionsearch-cli`), and is not always packaged on the AUR — the
/// `yay` half is only shown when [`AppSpec::gui_aur`] is set.
///
/// Returns `None` when the app ships no separate GUI package at all.
pub fn gui_install_hint(spec: &AppSpec) -> Option<String> {
    match (spec.gui_cargo, spec.gui_aur) {
        ("", "") => None,
        (cargo, "") => Some(format!("cargo install {cargo}")),
        ("", aur) => Some(format!("yay -S {aur}")),
        (cargo, aur) => Some(format!("cargo install {cargo}   (ou: yay -S {aur})")),
    }
}

const DEP_MPV: SysDep = SysDep {
    name: "mpv",
    label: "mpv (bin)",
    hint: "player binário do engine de optionMusic (pacman -S mpv)",
    required: true,
};
const DEP_LIBMPV: SysDep = SysDep {
    name: "libmpv",
    label: "libmpv",
    hint: "playback engine de optionMusic (pacman -S mpv)",
    required: true,
};
const DEP_CAVA: SysDep = SysDep {
    name: "cava",
    label: "cava",
    hint: "optional spectrum strip for optionMusic (pacman -S cava)",
    required: false,
};
const DEP_YTDLP: SysDep = SysDep {
    name: "yt-dlp",
    label: "yt-dlp",
    hint: "optional downloader for optionMusic (pacman -S yt-dlp)",
    required: false,
};
const DEP_FFMPEG: SysDep = SysDep {
    name: "ffmpeg",
    label: "ffmpeg",
    hint: "optional audio extraction for optionMusic (pacman -S ffmpeg)",
    required: false,
};
const DEP_XDG_OPEN: SysDep = SysDep {
    name: "xdg-open",
    label: "xdg-open",
    hint: "abrir arquivos/URLs de optionFiles (pacman -S xdg-utils)",
    required: true,
};
const DEP_IMAGEMAGICK: SysDep = SysDep {
    name: "imagemagick",
    label: "imagemagick",
    hint: "optional image previews for optionFiles (pacman -S imagemagick)",
    required: false,
};
const DEP_CLIPBOARD: SysDep = SysDep {
    name: "clipboard",
    label: "clipboard",
    hint: "optional copy-to-clipboard (pacman -S wl-clipboard | xclip)",
    required: false,
};
const DEP_GIO_TRASH: SysDep = SysDep {
    name: "gio",
    label: "gio/trash",
    hint: "optional trash instead of rm (pacman -S glib2)",
    required: false,
};
const DEP_GTK4: SysDep = SysDep {
    name: "gtk4",
    label: "GTK4",
    hint: "GUI toolkit for optionTerm (pacman -S gtk4)",
    required: true,
};
const DEP_ADW: SysDep = SysDep {
    name: "libadwaita",
    label: "libadwaita",
    hint: "Adwaita widgets for optionTerm (pacman -S libadwaita)",
    required: true,
};
const DEP_VTE: SysDep = SysDep {
    name: "vte-2.91",
    label: "VTE (GTK4)",
    hint: "terminal widget for optionTerm (pacman -S vte4)",
    required: true,
};
const DEP_PKGCONFIG: SysDep = SysDep {
    name: "pkg-config",
    label: "pkg-config",
    hint: "optional probe for GTK/VTE .pc files (pacman -S pkgconf)",
    required: false,
};
const DEP_EDITOR: SysDep = SysDep {
    name: "editor",
    label: "$EDITOR",
    hint: "editor used by optionFiles ($EDITOR ou vi/nano)",
    required: true,
};
const DEP_PDFTOTEXT: SysDep = SysDep {
    name: "pdftotext",
    label: "pdftotext",
    hint: "optional PDF text extraction for optionSearch previews (pacman -S poppler)",
    required: false,
};

const DEP_TERM_SET: &[SysDep] = &[DEP_GTK4, DEP_ADW, DEP_VTE, DEP_PKGCONFIG];
const DEP_FILES_SET: &[SysDep] = &[
    DEP_XDG_OPEN,
    DEP_IMAGEMAGICK,
    DEP_CLIPBOARD,
    DEP_GIO_TRASH,
    DEP_EDITOR,
];
const DEP_MUSIC_SET: &[SysDep] = &[DEP_MPV, DEP_LIBMPV, DEP_CAVA, DEP_YTDLP, DEP_FFMPEG];

/// Route table: canonical ids only. Aliases live in [`ALIASES`].
pub static APPS: &[AppSpec] = &[
    AppSpec {
        id: "files",
        mark: "◆",
        bins: &["optionfiles", "fls"],
        gui_bins: &["optionfiles-gtk", "fls-gtk"],
        about: "terminal file manager",
        cargo: "optionfiles",
        aur: "optionfiles",
        // The AUR package installs only optionfiles/fls, not the GTK bins.
        gui_cargo: "optionfiles-gui",
        gui_aur: "",
        deps: DEP_FILES_SET,
        is_gui: false,
    },
    AppSpec {
        id: "music",
        mark: "♪",
        bins: &["optionmusic", "msc"],
        gui_bins: &["optionmusic-gpui"],
        about: "CLI music player",
        cargo: "optionmusic",
        aur: "optionmusic",
        // The AUR package installs only optionmusic/msc, not the gpui bin.
        gui_cargo: "optionmusic-gpui",
        gui_aur: "",
        deps: DEP_MUSIC_SET,
        is_gui: false,
    },
    AppSpec {
        id: "cal",
        mark: "◷",
        bins: &["optioncalendar", "oca"],
        gui_bins: &[],
        about: "minimal local calendar",
        cargo: "optioncalendar",
        aur: "optioncalendar",
        gui_cargo: "",
        gui_aur: "",
        deps: &[],
        is_gui: false,
    },
    AppSpec {
        id: "terminal",
        mark: "◇",
        bins: &["optionterm"],
        gui_bins: &[],
        about: "GTK4 terminal with tiling splits",
        cargo: "optionterm",
        aur: "optionterm",
        // optionterm is itself the desktop app — see `is_gui`.
        gui_cargo: "",
        gui_aur: "",
        deps: DEP_TERM_SET,
        is_gui: true,
    },
    AppSpec {
        id: "opsh",
        mark: "❯",
        bins: &["opsh"],
        gui_bins: &[],
        about: "small local shell",
        cargo: "opsh",
        aur: "opsh",
        gui_cargo: "",
        gui_aur: "",
        deps: &[],
        is_gui: false,
    },
    AppSpec {
        id: "search",
        mark: "⌕",
        bins: &["optionsearch", "nld", "needle"],
        gui_bins: &["optionsearch-gtk", "needle"],
        about: "instant local file search",
        cargo: "optionsearch-cli",
        aur: "optionsearch",
        // The AUR package ships optionsearch-gtk alongside the CLI.
        gui_cargo: "optionsearch-gui",
        gui_aur: "optionsearch",
        deps: &[DEP_PDFTOTEXT],
        is_gui: false,
    },
];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lookup_known_ids() {
        assert_eq!(lookup("files").unwrap().bins, &["optionfiles", "fls"]);
        assert_eq!(lookup("music").unwrap().cargo, "optionmusic");
        assert_eq!(lookup("terminal").unwrap().bins, &["optionterm"]);
        assert_eq!(lookup("search").unwrap().cargo, "optionsearch-cli");
    }

    #[test]
    fn lookup_aliases_resolve_to_canonical() {
        assert_eq!(lookup("file").unwrap().id, "files");
        assert_eq!(lookup("f").unwrap().id, "files");
        assert_eq!(lookup("m").unwrap().id, "music");
        assert_eq!(lookup("c").unwrap().id, "cal");
        assert_eq!(lookup("s").unwrap().id, "search");
        assert_eq!(lookup("needle").unwrap().id, "search");
        assert_eq!(lookup("nld").unwrap().id, "search");
    }

    #[test]
    fn unknown_is_none() {
        assert!(lookup("nope").is_none());
        assert!(lookup("").is_none());
    }

    #[test]
    fn all_is_canonical_only() {
        // Aliases never appear in the canonical list.
        assert!(all().iter().all(|a| a.id != "file"
            && a.id != "f"
            && a.id != "m"
            && a.id != "c"
            && a.id != "s"
            && a.id != "needle"
            && a.id != "nld"));
        assert_eq!(all().len(), 6);
    }

    #[test]
    fn which_finds_on_path() {
        // `sh` is guaranteed present on any Unix build/CI runner.
        let found = which("sh");
        assert!(found.is_some());
        assert!(found.unwrap().is_file());
    }

    #[test]
    fn bin_override_wins() {
        // SAFETY: OPTION_BIN_FILES is read by no other test, so this
        // mutation cannot be observed by a test running in parallel, and it
        // is removed before returning.
        unsafe {
            std::env::set_var("OPTION_BIN_FILES", "/tmp/custom-files");
        }
        let spec = lookup("files").unwrap();
        assert_eq!(find_binary(spec), Some(PathBuf::from("/tmp/custom-files")));
        unsafe {
            std::env::remove_var("OPTION_BIN_FILES");
        }
    }

    #[test]
    fn gui_bins_registered() {
        assert_eq!(
            lookup("files").unwrap().gui_bins,
            &["optionfiles-gtk", "fls-gtk"]
        );
        assert_eq!(lookup("music").unwrap().gui_bins, &["optionmusic-gpui"]);
        assert_eq!(
            lookup("search").unwrap().gui_bins,
            &["optionsearch-gtk", "needle"]
        );
    }

    #[test]
    fn gui_hint_points_at_the_gui_package() {
        // The front-end lives in its own crate, not the CLI one.
        let search = lookup("search").unwrap();
        let hint = gui_install_hint(search).unwrap();
        assert!(hint.contains("optionsearch-gui"), "{hint}");
        // optionsearch (AUR) ships optionsearch-gtk, so offer it too.
        assert!(hint.contains("yay -S optionsearch"), "{hint}");
    }

    #[test]
    fn gui_hint_omits_aur_when_unpackaged() {
        // optionfiles/optionmusic (AUR) install only their CLI binaries.
        for id in ["files", "music"] {
            let hint = gui_install_hint(lookup(id).unwrap()).unwrap();
            assert!(!hint.contains("yay"), "{id}: {hint}");
            assert!(hint.starts_with("cargo install"), "{id}: {hint}");
        }
    }

    #[test]
    fn gui_hint_absent_without_frontend() {
        for id in ["cal", "opsh", "terminal"] {
            assert!(gui_install_hint(lookup(id).unwrap()).is_none(), "{id}");
        }
    }

    #[test]
    fn gui_hint_offers_aur_only_route() {
        // A front-end packaged on the AUR but not on crates.io still gets a
        // hint — the two fields are independent.
        let spec = AppSpec {
            gui_cargo: "",
            gui_aur: "optionsomething",
            ..*lookup("search").unwrap()
        };
        assert_eq!(
            gui_install_hint(&spec).as_deref(),
            Some("yay -S optionsomething")
        );
    }

    #[test]
    fn desktop_surface_includes_self_hosted_guis() {
        // optionTerm has no separate front-end, but `opt gui terminal`
        // works — so it must appear in the gui listing.
        let terminal = lookup("terminal").unwrap();
        assert!(has_desktop_surface(terminal));
        assert_eq!(desktop_bins(terminal), &["optionterm"]);

        // An app with a real front-end reports that front-end.
        let files = lookup("files").unwrap();
        assert!(has_desktop_surface(files));
        assert_eq!(desktop_bins(files), &["optionfiles-gtk", "fls-gtk"]);

        // CLI-only apps stay out of it.
        for id in ["cal", "opsh"] {
            let spec = lookup(id).unwrap();
            assert!(!has_desktop_surface(spec), "{id}");
            assert!(desktop_bins(spec).is_empty(), "{id}");
        }
    }

    #[test]
    fn marks_match_the_sdk() {
        // optionSDK owns the family marks; `opt` must never drift from it.
        // The path dependency resolves to the sibling checkout, so every
        // routed app is known to it.
        for spec in all() {
            let sdk = option_sdk::App::known(spec.id)
                .unwrap_or_else(|| panic!("SDK does not know {}", spec.id));
            assert_eq!(spec.mark, sdk.mark(), "{}", spec.id);
        }
    }

    #[test]
    fn override_is_trimmed() {
        // SAFETY: OPTION_BIN_CAL is read by no other test, so this mutation
        // cannot be observed by a test running in parallel, and it is
        // removed before returning.
        unsafe {
            std::env::set_var("OPTION_BIN_CAL", "  /tmp/spaced-cal  ");
        }
        let spec = lookup("cal").unwrap();
        assert_eq!(find_binary(spec), Some(PathBuf::from("/tmp/spaced-cal")));

        // An all-whitespace override counts as unset, so PATH lookup
        // resumes — whatever it finds, it is never the padded value.
        unsafe {
            std::env::set_var("OPTION_BIN_CAL", "   ");
        }
        assert_ne!(find_binary(spec), Some(PathBuf::from("   ")));
        unsafe {
            std::env::remove_var("OPTION_BIN_CAL");
        }
    }

    #[test]
    fn non_executable_file_is_not_a_binary() {
        // `which` must not report a leftover non-executable file as installed.
        // Checked through `is_executable` so the test never touches $PATH,
        // which other tests read in parallel.
        let dir = std::env::temp_dir().join(format!("opt-exec-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();

        let plain = dir.join("opt-not-executable");
        std::fs::write(&plain, b"#!/bin/sh\n").unwrap();
        assert!(!is_executable(&plain));

        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            std::fs::set_permissions(&plain, std::fs::Permissions::from_mode(0o755)).unwrap();
            assert!(is_executable(&plain));
        }

        // A directory on PATH is never a runnable binary either.
        assert!(!is_executable(&dir));
        assert!(!is_executable(&dir.join("does-not-exist")));

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn gui_bins_empty_without_frontend() {
        // cal/opsh have no GUI; terminal's only bin already is one.
        for id in ["cal", "opsh", "terminal"] {
            assert!(lookup(id).unwrap().gui_bins.is_empty());
        }
    }

    #[test]
    fn gui_is_not_an_app_id() {
        // `gui` is a reserved verb (like status/doctor), never an app id.
        assert!(lookup("gui").is_none());
    }

    #[test]
    fn search_metadata() {
        // crates.io `needle` is a third-party crate; ours is `optionsearch-cli`.
        // `nld`/`needle` stay as compat bins for the rename.
        let search = lookup("search").unwrap();
        assert_eq!(search.bins, &["optionsearch", "nld", "needle"]);
        assert_eq!(search.gui_bins, &["optionsearch-gtk", "needle"]);
        assert_eq!(search.cargo, "optionsearch-cli");
        assert_eq!(search.aur, "optionsearch");
        assert_eq!(lookup("needle").unwrap().id, "search");
    }

    #[test]
    fn gui_bin_override_wins() {
        // SAFETY: OPTION_GUI_BIN_FILES is read by no other test, so this
        // mutation cannot be observed by a test running in parallel, and it
        // is removed before returning.
        unsafe {
            std::env::set_var("OPTION_GUI_BIN_FILES", "/tmp/custom-fls-gtk");
        }
        let spec = lookup("files").unwrap();
        assert_eq!(
            find_gui_binary(spec),
            Some(PathBuf::from("/tmp/custom-fls-gtk"))
        );
        unsafe {
            std::env::remove_var("OPTION_GUI_BIN_FILES");
        }
    }

    #[test]
    fn deps_present_for_gui_apps() {
        assert!(!lookup("music").unwrap().deps.is_empty());
        assert!(!lookup("terminal").unwrap().deps.is_empty());
        assert!(lookup("opsh").unwrap().deps.is_empty());
    }

    #[test]
    fn required_flags_are_set() {
        let files = lookup("files").unwrap();
        assert!(
            files
                .deps
                .iter()
                .find(|d| d.name == "xdg-open")
                .is_some_and(|d| d.required)
        );
        assert!(
            files
                .deps
                .iter()
                .find(|d| d.name == "imagemagick")
                .is_some_and(|d| !d.required)
        );
        let music = lookup("music").unwrap();
        assert!(
            music
                .deps
                .iter()
                .find(|d| d.name == "libmpv")
                .is_some_and(|d| d.required)
        );
        assert!(
            music
                .deps
                .iter()
                .find(|d| d.name == "cava")
                .is_some_and(|d| !d.required)
        );
    }

    #[test]
    fn dep_present_plain_binary() {
        let present = SysDep {
            name: "sh",
            label: "sh",
            hint: "test",
            required: true,
        };
        assert!(dep_present(&present));
        let absent = SysDep {
            name: "opt-totally-missing-bin-xyz",
            label: "missing",
            hint: "test",
            required: false,
        };
        assert!(!dep_present(&absent));
    }

    #[test]
    fn files_music_have_no_raw_lib_names() {
        // gtk/adwaita/vte must be detected via pkg-config, never has_binary.
        for spec in [lookup("files").unwrap(), lookup("terminal").unwrap()] {
            for dep in spec.deps {
                assert_ne!(dep.name, "gtk4-lib-check");
            }
        }
    }
}
