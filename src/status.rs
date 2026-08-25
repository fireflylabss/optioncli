//! `opt status` — list family apps, their installed state and versions.

use crate::apps::{AppSpec, all, binary_version, find_binary};
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
                "· {:<9} {}  ({})",
                styled(spec.id, color),
                styled(&version, color),
                styled(spec.bins[0], color)
            );
        }
        None => {
            let _ = writeln!(
                out,
                "· {:<9} {}  ({})",
                styled(spec.id, color),
                styled("—", color),
                styled("não instalado", color)
            );
        }
    }
}

/// Wrap `text` in ANSI bold when `on` is true, else return it unchanged.
fn styled(text: &str, on: bool) -> String {
    if on {
        format!("\x1b[1m{text}\x1b[0m")
    } else {
        text.to_string()
    }
}
