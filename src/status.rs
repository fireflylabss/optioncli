//! `opt status` — list family apps, their installed state and versions.

use crate::apps::{AppSpec, all, binary_version, find_binary, find_gui_binary};
use std::io::{self, IsTerminal, Write};

/// Print the status report for every known app.
pub fn report() {
    let color = io::stdout().is_terminal() && option_sdk::color_enabled();
    let mut out = io::stdout().lock();

    let _ = writeln!(
        out,
        "◆ {}  {}",
        styled("opt", color),
        styled(env!("CARGO_PKG_VERSION"), color)
    );

    for spec in all() {
        print_spec(&mut out, spec, color);
    }
}

fn print_spec(out: &mut impl Write, spec: &AppSpec, color: bool) {
    let gui = gui_suffix(spec, color);
    match find_binary(spec) {
        Some(bin) => {
            // GUI apps (e.g. optionTerm) launch their window on `--version`
            // instead of printing a version, so we skip running them.
            let version = if spec.is_gui {
                "—".to_string()
            } else {
                binary_version(&bin).unwrap_or_else(|| "—".to_string())
            };
            let _ = writeln!(
                out,
                "· {:<9} {}  ({}){}",
                styled(spec.id, color),
                styled(&version, color),
                styled(spec.bins[0], color),
                gui
            );
        }
        None => {
            let _ = writeln!(
                out,
                "· {:<9} {}  ({}){}",
                styled(spec.id, color),
                styled("—", color),
                styled("não instalado", color),
                gui
            );
        }
    }
}

/// ` + gui (<bin>)` suffix when the app has a desktop front-end configured
/// (`gui_bins`), reporting the bin found or `não instalado`.
fn gui_suffix(spec: &AppSpec, color: bool) -> String {
    if spec.gui_bins.is_empty() {
        return String::new();
    }
    let label = match find_gui_binary(spec) {
        Some(bin) => bin
            .file_name()
            .map(|name| name.to_string_lossy().into_owned())
            .unwrap_or_else(|| bin.display().to_string()),
        None => "não instalado".to_string(),
    };
    format!(" + gui ({})", styled(&label, color))
}

/// Wrap `text` in ANSI bold when `on` is true, else return it unchanged.
fn styled(text: &str, on: bool) -> String {
    if on {
        format!("\x1b[1m{text}\x1b[0m")
    } else {
        text.to_string()
    }
}
