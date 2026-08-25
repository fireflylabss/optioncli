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
    /// Executable or library name to detect on the system.
    pub name: &'static str,
    /// Short human label shown in `doctor`.
    pub label: &'static str,
    /// One-line why / how to install.
    pub hint: &'static str,
}

/// The resolved identity of one family app.
#[derive(Debug, Clone, Copy)]
pub struct AppSpec {
    /// Canonical id used on the command line, e.g. `music`.
    pub id: &'static str,
    /// One-char mark used by optionSDK (\u{25c6} ◇ ♪ ⌕ …).
    pub mark: &'static str,
    /// Binary names to try, in order. The first one present on `PATH` wins.
    pub bins: &'static [&'static str],
    /// What the app does, for the menu / help.
    pub about: &'static str,
    /// Cargo package name (`cargo install <cargo>`).
    pub cargo: &'static str,
    /// AUR package name (`yay -S <aur>`).
    pub aur: &'static str,
    /// System binary dependencies checked by `opt doctor`.
    pub deps: &'static [SysDep],
    /// True for desktop/GUI apps whose `--version` launches the app instead
    /// of printing a version (e.g. optionTerm). `status` skips running them.
    pub is_gui: bool,
}

/// Short aliases accepted on the command line, mapping to a canonical id.
const ALIASES: &[(&str, &str)] = &[("f", "files"), ("file", "files"), ("m", "music")];

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
    label: "mpv / libmpv",
    hint: "playback engine for optionMusic (pacman -S mpv)",
};
const DEP_CAVA: SysDep = SysDep {
    name: "cava",
    label: "cava",
    hint: "optional spectrum strip for optionMusic (pacman -S cava)",
};
const DEP_YTDLP: SysDep = SysDep {
    name: "yt-dlp",
    label: "yt-dlp",
    hint: "optional downloader for optionMusic (pacman -S yt-dlp)",
};
const DEP_FFMPEG: SysDep = SysDep {
    name: "ffmpeg",
    label: "ffmpeg",
    hint: "optional audio extraction for optionMusic (pacman -S ffmpeg)",
};
const DEP_MAGICK: SysDep = SysDep {
    name: "magick",
    label: "imagemagick",
    hint: "optional image previews for optionFiles (pacman -S imagemagick)",
};
const DEP_GTK4: SysDep = SysDep {
    name: "gtk4",
    label: "GTK4",
    hint: "GUI toolkit for optionTerm (pacman -S gtk4)",
};
const DEP_ADW: SysDep = SysDep {
    name: "libadwaita",
    label: "libadwaita",
    hint: "Adwaita widgets for optionTerm (pacman -S libadwaita)",
};
const DEP_VTE: SysDep = SysDep {
    name: "vte-2.91",
    label: "VTE (GTK4)",
    hint: "terminal widget for optionTerm (pacman -S vte4)",
};
const DEP_EDITOR: SysDep = SysDep {
    name: "nano",
    label: "$EDITOR",
    hint: "editor used by optionFiles / optionNotes (any dev editor)",
};
const DEP_PDFTOTEXT: SysDep = SysDep {
    name: "pdftotext",
    label: "pdftotext",
    hint: "optional PDF text extraction for Needle previews (pacman -S poppler)",
};

const DEP_TERM_SET: &[SysDep] = &[DEP_GTK4, DEP_ADW, DEP_VTE];

/// Route table: canonical ids only. Aliases live in [`ALIASES`].
pub static APPS: &[AppSpec] = &[
    AppSpec {
        id: "files",
        mark: "\u{25c6}",
        bins: &["optionfiles", "fls"],
        about: "terminal file manager",
        cargo: "optionfiles",
        aur: "optionfiles",
        deps: &[DEP_MAGICK, DEP_EDITOR],
        is_gui: false,
    },
    AppSpec {
        id: "music",
        mark: "\u{266a}",
        bins: &["optionmusic", "msc"],
        about: "CLI music player",
        cargo: "optionmusic",
        aur: "optionmusic",
        deps: &[DEP_MPV, DEP_CAVA, DEP_YTDLP, DEP_FFMPEG],
        is_gui: false,
    },
    AppSpec {
        id: "notes",
        mark: "\u{25c7}",
        bins: &["nts"],
        about: "local-first markdown notes",
        cargo: "optionnotes",
        aur: "optionnotes",
        deps: &[DEP_EDITOR],
        is_gui: false,
    },
    AppSpec {
        id: "terminal",
        mark: "\u{25c7}",
        bins: &["optionterm"],
        about: "GTK4 terminal with tiling splits",
        cargo: "optionterm",
        aur: "optionterm",
        deps: DEP_TERM_SET,
        is_gui: true,
    },
    AppSpec {
        id: "opsh",
        mark: "\u{25c6}",
        bins: &["opsh"],
        about: "small local shell",
        cargo: "opsh",
        aur: "opsh",
        deps: &[],
        is_gui: false,
    },
    AppSpec {
        id: "fat",
        mark: "\u{25c6}",
        bins: &["fat"],
        about: "fast syntax-aware cat",
        cargo: "ofat",
        aur: "ofat",
        deps: &[],
        is_gui: false,
    },
    AppSpec {
        id: "needle",
        mark: "\u{2315}",
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
                .all(|a| a.id != "file" && a.id != "f" && a.id != "m")
        );
        assert_eq!(all().len(), 7);
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
}
