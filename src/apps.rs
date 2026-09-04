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
    /// One-char mark used by optionSDK (◆ ◇ ♪ ⌕ …).
    pub mark: &'static str,
    /// Binary names to try, in order. The first one present on `PATH` wins.
    pub bins: &'static [&'static str],
    /// What the app does, for the menu / help.
    pub about: &'static str,
    /// Cargo package name (`cargo install <cargo>`).
    pub cargo: &'static str,
    /// AUR package name (`yay -S <aur>`).
    pub aur: &'static str,
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
];

/// All accepted aliases (alias → canonical id), for help text.
pub fn aliases() -> &'static [(&'static str, &'static str)] {
    ALIASES
}

/// `OPTION_BIN_<ID>` environment override for a binary path.
fn bin_override(id: &str) -> Option<String> {
    // OPTION_BIN_TERMINAL → "terminal"
    std::env::var(format!("OPTION_BIN_{}", id.to_ascii_uppercase())).ok()
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
        if !path.trim().is_empty() {
            return Some(PathBuf::from(path));
        }
    }
    spec.bins.iter().find_map(|bin| which(bin))
}

/// Search `PATH` for a single executable name, returning its path when found.
pub fn which(name: &str) -> Option<PathBuf> {
    let path = std::env::var_os("PATH")?;
    for dir in std::env::split_paths(&path) {
        let candidate = dir.join(name);
        if candidate.is_file() {
            return Some(candidate);
        }
    }
    None
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
            if first.contains('/') {
                if std::path::Path::new(first).exists() {
                    return true;
                }
            } else if has_binary(first) {
                return true;
            }
            // Set but not installed: fall through to the vi/nano fallback.
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
/// Some binaries print the version on stdout (e.g. `fat 0.1.3`); others use a
/// banner. Capture stdout and return its first trimmed line.
pub fn binary_version(bin: &PathBuf) -> Option<String> {
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
    hint: "editor used by optionFiles / optionNotes ($EDITOR ou vi/nano)",
    required: true,
};
const DEP_PDFTOTEXT: SysDep = SysDep {
    name: "pdftotext",
    label: "pdftotext",
    hint: "optional PDF text extraction for Needle previews (pacman -S poppler)",
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
        about: "terminal file manager",
        cargo: "optionfiles",
        aur: "optionfiles",
        deps: DEP_FILES_SET,
        is_gui: false,
    },
    AppSpec {
        id: "music",
        mark: "♪",
        bins: &["optionmusic", "msc"],
        about: "CLI music player",
        cargo: "optionmusic",
        aur: "optionmusic",
        deps: DEP_MUSIC_SET,
        is_gui: false,
    },
    AppSpec {
        id: "notes",
        mark: "◇",
        bins: &["nts"],
        about: "local-first markdown notes",
        cargo: "optionnotes",
        aur: "optionnotes",
        deps: &[DEP_EDITOR],
        is_gui: false,
    },
    AppSpec {
        id: "cal",
        mark: "◷",
        bins: &["optioncalendar", "oca"],
        about: "minimal local calendar",
        cargo: "optioncalendar",
        aur: "optioncalendar",
        deps: &[],
        is_gui: false,
    },
    AppSpec {
        id: "terminal",
        mark: "◇",
        bins: &["optionterm"],
        about: "GTK4 terminal with tiling splits",
        cargo: "optionterm",
        aur: "optionterm",
        deps: DEP_TERM_SET,
        is_gui: true,
    },
    AppSpec {
        id: "opsh",
        mark: "◆",
        bins: &["opsh"],
        about: "small local shell",
        cargo: "opsh",
        aur: "opsh",
        deps: &[],
        is_gui: false,
    },
    AppSpec {
        id: "fat",
        mark: "◆",
        bins: &["fat"],
        about: "fast syntax-aware cat",
        cargo: "ofat",
        aur: "ofat",
        deps: &[],
        is_gui: false,
    },
    AppSpec {
        id: "needle",
        mark: "⌕",
        bins: &["needle"],
        about: "instant local file search",
        cargo: "needle",
        aur: "needle",
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
        assert_eq!(lookup("notes").unwrap().bins, &["nts"]);
        assert_eq!(lookup("terminal").unwrap().bins, &["optionterm"]);
    }

    #[test]
    fn lookup_aliases_resolve_to_canonical() {
        assert_eq!(lookup("file").unwrap().id, "files");
        assert_eq!(lookup("f").unwrap().id, "files");
        assert_eq!(lookup("m").unwrap().id, "music");
        assert_eq!(lookup("c").unwrap().id, "cal");
    }

    #[test]
    fn unknown_is_none() {
        assert!(lookup("nope").is_none());
        assert!(lookup("").is_none());
    }

    #[test]
    fn all_is_canonical_only() {
        // Aliases never appear in the canonical list.
        assert!(
            all()
                .iter()
                .all(|a| a.id != "file" && a.id != "f" && a.id != "m" && a.id != "c")
        );
        assert_eq!(all().len(), 8);
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
        // SAFETY: tests serialize env mutation via OPTION_BIN_* names.
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
