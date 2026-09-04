//! `opt doctor` — check each app's binary and system dependencies.

use crate::apps::{all, alt_hint, dep_present, find_binary};
use std::io::{self, IsTerminal, Write};

/// Print a per-app health report: binary present, then each system dep.
///
/// Visual only (always succeeds): required deps missing show as
/// `faltando (req)` with install hints; optional ones as `ausente (opc)`.
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
            let present = dep_present(dep);
            let tag = if dep.required { "req" } else { "opc" };
            if present {
                let _ = writeln!(
                    out,
                    "  {:>10}  {}  {}",
                    dim(dep.label, color),
                    ok("ok", color),
                    dim(&format!("[{tag}]"), color)
                );
            } else if dep.required {
                let (apt, dnf) = alt_hint(dep.name);
                let _ = writeln!(
                    out,
                    "  {:>10}  {}  {}",
                    dim(dep.label, color),
                    missing("faltando (req)", color),
                    dim(&format!("[{tag}]"), color)
                );
                let _ = writeln!(out, "  {:>10}  {}", dim("", color), dim(dep.hint, color));
                let _ = writeln!(
                    out,
                    "  {:>10}  {}",
                    dim("", color),
                    dim(&format!("apt: {apt} · dnf: {dnf}"), color)
                );
            } else {
                let (apt, dnf) = alt_hint(dep.name);
                let _ = writeln!(
                    out,
                    "  {:>10}  {}  {}",
                    dim(dep.label, color),
                    missing("ausente (opc)", color),
                    dim(&format!("[{tag}]"), color)
                );
                let _ = writeln!(out, "  {:>10}  {}", dim("", color), dim(dep.hint, color));
                let _ = writeln!(
                    out,
                    "  {:>10}  {}",
                    dim("", color),
                    dim(&format!("apt: {apt} · dnf: {dnf}"), color)
                );
            }
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
