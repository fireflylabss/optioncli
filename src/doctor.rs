//! `opt doctor` — check each app's binary and system dependencies.

use crate::apps::{all, find_binary, has_binary};
use std::io::{self, IsTerminal, Write};

/// Print a per-app health report: binary present, then each system dep.
pub fn report() {
    let color = io::stdout().is_terminal() && option_sdk::color_enabled();
    let mut out = io::stdout().lock();

    let _ = writeln!(
        out,
        "◆ {} {}",
        bold("opt doctor", color),
        dim("· local first", color)
    );

    for spec in all() {
        let _ = writeln!(out, "\n{} {}", spec.mark, bold(spec.id, color));

        let bin_ok = find_binary(spec).is_some();
        let _ = writeln!(
            out,
            "  {:>10}  {}",
            dim("binário", color),
            if bin_ok {
                ok("ok", color)
            } else {
                missing(&format!("faltando — {}", spec.bins[0]), color)
            }
        );

        for dep in spec.deps {
            let present = has_binary(dep.name);
            let _ = writeln!(
                out,
                "  {:>10}  {}  {}",
                dim(dep.label, color),
                if present {
                    ok("ok", color)
                } else {
                    missing("faltando", color)
                },
                dim(dep.hint, color)
            );
        }
    }
}

fn bold(text: &str, on: bool) -> String {
    if on {
        format!("\x1b[1m{text}\x1b[0m")
    } else {
        text.to_string()
    }
}

fn dim(text: &str, on: bool) -> String {
    if on {
        format!("\x1b[2m{text}\x1b[0m")
    } else {
        text.to_string()
    }
}

fn ok(text: &str, on: bool) -> String {
    if on {
        format!("\x1b[32m{text}\x1b[0m")
    } else {
        text.to_string()
    }
}

fn missing(text: &str, on: bool) -> String {
    if on {
        format!("\x1b[31m{text}\x1b[0m")
    } else {
        text.to_string()
    }
}
