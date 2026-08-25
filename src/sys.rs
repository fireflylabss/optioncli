//! `opt sys` — small local system utilities.
//!
//! `opt sys clean` removes each app's `~/.option/<id>/cache`; `opt sys info`
//! prints OS / arch / option root; `opt sys path` lists `PATH` entries.

use crate::apps::all;
use std::io::{self, IsTerminal, Write};
use std::process::ExitCode;

/// Dispatch a `sys` subcommand. No args prints help.
pub fn dispatch(args: &[String]) -> ExitCode {
    let Some(command) = args.first() else {
        print_help();
        return ExitCode::SUCCESS;
    };
    match command.as_str() {
        "-h" | "--help" | "help" => {
            print_help();
            ExitCode::SUCCESS
        }
        "clean" => clean(),
        "info" => info(),
        "path" => path(),
        other => {
            eprintln!("opt sys: '{other}' não é um utilitário conhecido.");
            eprintln!("Utilitários: clean, info, path");
            ExitCode::FAILURE
        }
    }
}

/// Remove every app's cache directory under the Option root.
fn clean() -> ExitCode {
    let color = color();
    let mut out = io::stdout().lock();
    let _ = writeln!(out, "{}", bold("limpar caches ~/.option", color));

    let removed = all()
        .iter()
        .filter_map(|spec| {
            let cache = option_sdk::App::known(spec.id)
                .map(|app| app.cache_dir())
                .unwrap_or_else(|| {
                    // Apps not in the SDK (e.g. needle) still use <id>/cache.
                    option_sdk::option_root().join(spec.id).join("cache")
                });
            if cache.is_dir() {
                match std::fs::remove_dir_all(&cache) {
                    Ok(()) => Some(cache),
                    Err(e) => {
                        let _ = writeln!(
                            out,
                            "  {}  {}  {}",
                            dim(spec.id, color),
                            missing("erro", color),
                            dim(&e.to_string(), color)
                        );
                        None
                    }
                }
            } else {
                None
            }
        })
        .count();

    let _ = writeln!(out, "  removidos: {} cache(s)", removed);
    ExitCode::SUCCESS
}

/// Print OS, architecture, home and the Option root.
fn info() -> ExitCode {
    let color = color();
    let mut out = io::stdout().lock();

    let _ = writeln!(out, "{}", bold("info do sistema", color));
    let _ = writeln!(out, "  {:<10} {}", dim("os", color), std::env::consts::OS);
    let _ = writeln!(
        out,
        "  {:<10} {}",
        dim("arch", color),
        std::env::consts::ARCH
    );
    let _ = writeln!(
        out,
        "  {:<10} {}",
        dim("home", color),
        option_sdk::home_dir().display()
    );
    let _ = writeln!(
        out,
        "  {:<10} {}",
        dim("option", color),
        option_sdk::option_root().display()
    );
    let _ = writeln!(
        out,
        "  {:<10} {}",
        dim("no-color", color),
        if option_sdk::color_enabled() {
            "não"
        } else {
            "sim"
        }
    );
    ExitCode::SUCCESS
}

/// Print each `PATH` entry, one per line.
fn path() -> ExitCode {
    let mut out = io::stdout().lock();
    if let Some(paths) = std::env::var_os("PATH") {
        for dir in std::env::split_paths(&paths) {
            let _ = writeln!(out, "{}", dir.display());
        }
    }
    ExitCode::SUCCESS
}

fn print_help() {
    println!("opt sys — local system utilities");
    println!();
    println!("USAGE:");
    println!("    opt sys clean    remove each app's ~/.option/<id>/cache");
    println!("    opt sys info     show OS, arch, home and option root");
    println!("    opt sys path     list PATH entries");
}

fn color() -> bool {
    io::stdout().is_terminal() && option_sdk::color_enabled()
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

fn missing(text: &str, on: bool) -> String {
    if on {
        format!("\x1b[31m{text}\x1b[0m")
    } else {
        text.to_string()
    }
}
