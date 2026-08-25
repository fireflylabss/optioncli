//! `opt install` / `opt update` — install or update Option family apps.
//!
//! Defaults to `cargo install` (local, no sudo). Honors `OPTION_PKG=yay` to use
//! the AUR instead. `opt install` with no args installs the whole family.

use crate::apps::{AppSpec, all, lookup};
use std::process::{Command, ExitCode};

/// Install all family apps with the chosen package manager.
pub fn install_all() {
    let pm = PackageManager::detect();
    for spec in all() {
        install_one(spec, pm, "install");
    }
}

/// Install or update the listed apps, validating ids first.
pub fn install_many(ids: &[String]) -> ExitCode {
    let pm = PackageManager::detect();
    let mut ok = true;
    for id in ids {
        match lookup(id) {
            Some(spec) => install_one(spec, pm, "install"),
            None => {
                eprintln!("opt: '{id}' não é um app Option conhecido.");
                ok = false;
            }
        }
    }
    if ok {
        ExitCode::SUCCESS
    } else {
        ExitCode::FAILURE
    }
}

fn install_one(spec: &AppSpec, pm: PackageManager, verb: &str) {
    match pm {
        PackageManager::Cargo => {
            println!("==> cargo {verb} {}", spec.cargo);
            let status = Command::new("cargo").arg(verb).arg(&spec.cargo).status();
            match status {
                Ok(status) if status.success() => println!("    ok {}", spec.id),
                Ok(status) => {
                    eprintln!("    falhou (exit {})", status.code().unwrap_or(1));
                }
                Err(e) => eprintln!("    erro: {e}"),
            }
        }
        PackageManager::Yay => {
            println!("==> yay -S {}", spec.aur);
            let status = Command::new("yay")
                .args(["-S", "--noconfirm", spec.aur])
                .status();
            match status {
                Ok(status) if status.success() => println!("    ok {}", spec.id),
                Ok(status) => {
                    eprintln!("    falhou (exit {})", status.code().unwrap_or(1));
                }
                Err(e) => eprintln!("    erro: {e}"),
            }
        }
    }
}

/// How to install: cargo by default, or yay when `OPTION_PKG=yay`.
#[derive(Clone, Copy)]
enum PackageManager {
    Cargo,
    Yay,
}

impl PackageManager {
    fn detect() -> Self {
        match std::env::var("OPTION_PKG").as_deref() {
            Ok("yay") | Ok("pacman") => Self::Yay,
            _ => Self::Cargo,
        }
    }
}
