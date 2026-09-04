//! `opt install` / `opt update` — install or update Option family apps.
//!
//! Defaults to `cargo install` (local, no sudo). Honors
//! `OPTION_PKG=yay|paru|pacman` to use Arch packages instead.
//! `opt install` with no args installs the whole family; `opt update`
//! reinstalls with `cargo install --force` (cargo) or refresh-upgrades
//! the family AUR packages (`<helper> -Syu --noconfirm <pkgs...>`).
//! `opt install family` installs the `option-family` AUR metapackage
//! (all apps in one transaction) when using an Arch helper.

use crate::apps::{AppSpec, all, lookup};
use std::process::{Command, ExitCode};

/// AUR metapackage pulling the whole family in one transaction.
const FAMILY_META: &str = "option-family";

/// Install all family apps with the chosen package manager.
pub fn install_all() {
    install_all_verb("install");
}

/// Update all family apps with the chosen package manager.
pub fn update_all() {
    install_all_verb("update");
}

fn install_all_verb(verb: &str) {
    let pm = PackageManager::detect();
    if verb == "update" && pm.is_arch() {
        update_arch_family(pm);
        return;
    }
    for spec in all() {
        install_one(spec, pm, verb);
    }
}

/// Install or update the listed apps, validating ids first.
///
/// The special id `family` installs the `option-family` metapackage
/// on Arch helpers, or every app via cargo otherwise.
pub fn install_many(ids: &[String], verb: &str) -> ExitCode {
    let pm = PackageManager::detect();
    if ids.iter().any(|id| id == "family") {
        install_family_meta(pm, verb);
        // Keep processing any other ids given alongside `family`.
        let rest: Vec<String> = ids.iter().filter(|id| *id != "family").cloned().collect();
        if rest.is_empty() {
            return ExitCode::SUCCESS;
        }
        return install_many(&rest, verb);
    }
    let mut ok = true;
    for id in ids {
        match lookup(id) {
            Some(spec) => install_one(spec, pm, verb),
            None => {
                eprintln!("opt: '{id}' não é um app Option conhecido.");
                eprintln!("      (dica: 'family' instala o metapacote option-family no AUR)");
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

fn install_family_meta(pm: PackageManager, verb: &str) {
    match pm {
        PackageManager::Cargo => {
            println!("==> cargo {verb} (família: cada app)");
            for spec in all() {
                install_one(spec, pm, verb);
            }
        }
        _ => {
            let helper = pm.helper_bin();
            if verb == "update" {
                println!("==> {helper} -Syu --noconfirm {FAMILY_META}");
                run_helper_update(pm, &[FAMILY_META.to_string()]);
            } else {
                println!("==> {helper} -S --noconfirm {FAMILY_META}");
                let status = Command::new(helper)
                    .args(["-S", "--noconfirm", FAMILY_META])
                    .status();
                match status {
                    Ok(status) if status.success() => println!("    ok family"),
                    Ok(status) => {
                        eprintln!("    falhou (exit {})", status.code().unwrap_or(1));
                    }
                    Err(e) => eprintln!("    erro: {e}"),
                }
            }
        }
    }
}

/// Refresh-upgrade the whole family via an Arch helper.
fn update_arch_family(pm: PackageManager) {
    let pkgs: Vec<String> = all().iter().map(|s| s.aur.to_string()).collect();
    let helper = pm.helper_bin();
    println!("==> {helper} -Syu --noconfirm {}", pkgs.join(" "));
    run_helper_update(pm, &pkgs);
}

fn run_helper_update(pm: PackageManager, pkgs: &[String]) {
    let helper = pm.helper_bin();
    let mut cmd = Command::new(helper);
    if matches!(pm, PackageManager::Pacman) {
        cmd.arg("-Syu").arg("--noconfirm");
    } else {
        cmd.arg("-Syu").arg("--noconfirm");
    }
    cmd.args(pkgs);
    match cmd.status() {
        Ok(status) if status.success() => println!("    ok (atualizado)"),
        Ok(status) => eprintln!("    falhou (exit {})", status.code().unwrap_or(1)),
        Err(e) => eprintln!("    erro: {e}"),
    }
}

fn install_one(spec: &AppSpec, pm: PackageManager, verb: &str) {
    match pm {
        PackageManager::Cargo => {
            if verb == "update" {
                println!("==> cargo install --force {}", spec.cargo);
                let status = Command::new("cargo")
                    .args(["install", "--force", spec.cargo])
                    .status();
                match status {
                    Ok(status) if status.success() => println!("    ok {}", spec.id),
                    Ok(status) => {
                        eprintln!("    falhou (exit {})", status.code().unwrap_or(1));
                    }
                    Err(e) => eprintln!("    erro: {e}"),
                }
            } else {
                println!("==> cargo install {}", spec.cargo);
                let status = Command::new("cargo").args(["install", spec.cargo]).status();
                match status {
                    Ok(status) if status.success() => println!("    ok {}", spec.id),
                    Ok(status) => {
                        eprintln!("    falhou (exit {})", status.code().unwrap_or(1));
                    }
                    Err(e) => eprintln!("    erro: {e}"),
                }
            }
        }
        PackageManager::Yay | PackageManager::Paru => {
            let helper = pm.helper_bin();
            if verb == "update" {
                println!("==> {helper} -Syu --noconfirm {}", spec.aur);
                run_helper_update(pm, &[spec.aur.to_string()]);
            } else {
                println!("==> {helper} -S --noconfirm {}", spec.aur);
                let status = Command::new(helper)
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
        PackageManager::Pacman => {
            if verb == "update" {
                println!("==> sudo pacman -Syu --noconfirm {}", spec.aur);
                run_helper_update(pm, &[spec.aur.to_string()]);
            } else {
                println!("==> sudo pacman -S --noconfirm {}", spec.aur);
                let status = Command::new("sudo")
                    .args(["pacman", "-S", "--noconfirm", spec.aur])
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
}

/// How to install: cargo by default, or an Arch helper via `OPTION_PKG`.
#[derive(Clone, Copy)]
enum PackageManager {
    Cargo,
    Yay,
    Paru,
    Pacman,
}

impl PackageManager {
    fn detect() -> Self {
        match std::env::var("OPTION_PKG").as_deref() {
            Ok("yay") => Self::Yay,
            Ok("paru") => Self::Paru,
            Ok("pacman") => Self::Pacman,
            Ok("aur") => {
                if crate::apps::has_binary("paru") {
                    Self::Paru
                } else {
                    Self::Yay
                }
            }
            _ => Self::Cargo,
        }
    }

    fn is_arch(&self) -> bool {
        !matches!(self, Self::Cargo)
    }

    fn helper_bin(&self) -> &'static str {
        match self {
            Self::Cargo => "cargo",
            Self::Yay => "yay",
            Self::Paru => "paru",
            Self::Pacman => "pacman",
        }
    }
}
