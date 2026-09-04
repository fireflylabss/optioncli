//! opt — the Option family CLI.
//!
//! `opt <app> [args...]` forwards to the matching app binary. It also offers
//! family-level commands: `status`, `doctor`, `install`, `sys` and `version`.
//! It does not implement any app logic itself; it only knows how to find,
//! install and check each app.

mod apps;
mod doctor;
mod install;
mod run;
mod status;
mod sys;

use std::env;
use std::process::ExitCode;

use apps::{aliases, all, find_binary, install_hint, lookup};
use run::run;

const MENU_HEADER: &str = "◆ opt — the Option family";

fn main() -> ExitCode {
    let args: Vec<String> = env::args().skip(1).collect();

    // `opt -- music --stuff`: the `--` lets a following token be an app id
    // even if it could be mistaken for a global flag.
    let mut args = args;
    if args.first().is_some_and(|a| a == "--") {
        args.remove(0);
    }

    if args.is_empty() {
        print_menu();
        return ExitCode::SUCCESS;
    }

    match args[0].as_str() {
        "-h" | "--help" | "help" => {
            print_help();
            ExitCode::SUCCESS
        }
        "-V" | "--version" | "version" => {
            println!("opt {}", env!("CARGO_PKG_VERSION"));
            ExitCode::SUCCESS
        }
        "status" => {
            status::report();
            ExitCode::SUCCESS
        }
        "doctor" => {
            doctor::report();
            ExitCode::SUCCESS
        }
        "install" => {
            // `opt install` (all) or `opt install <app>...`
            let rest = &args[1..];
            if rest.is_empty() {
                install::install_all();
                ExitCode::SUCCESS
            } else {
                install::install_many(rest, "install")
            }
        }
        "update" => {
            // `opt update` really updates: cargo --force / helper -Syu.
            let rest = &args[1..];
            if rest.is_empty() {
                install::update_all();
                ExitCode::SUCCESS
            } else {
                install::install_many(rest, "update")
            }
        }
        "sys" => sys::dispatch(&args[1..]),
        app_id => dispatch(app_id, &args[1..]),
    }
}

/// Run the app matching `app_id`, forwarding the rest of the arguments.
fn dispatch(app_id: &str, rest: &[String]) -> ExitCode {
    let Some(spec) = lookup(app_id) else {
        eprintln!("opt: '{app_id}' não é um app Option conhecido.");
        eprintln!("Apps: {}", known_ids());
        return ExitCode::FAILURE;
    };

    match find_binary(spec) {
        Some(bin) => match run(&bin, rest) {
            Ok(code) => code,
            Err(error) => {
                eprintln!("opt: {error}");
                ExitCode::FAILURE
            }
        },
        None => {
            eprintln!("opt: '{}' não encontrado.", spec.id);
            eprintln!("       Instale com: {}", install_hint(spec));
            ExitCode::FAILURE
        }
    }
}

fn known_ids() -> String {
    all().iter().map(|s| s.id).collect::<Vec<_>>().join(", ")
}

fn print_menu() {
    println!();
    println!("{MENU_HEADER}");
    println!();
    print_routing_table();
    println!();
    println!("  status      apps instalados + versões");
    println!("  doctor      dependências de sistema de cada app");
    println!("  install     instala a família (ou um app)");
    println!("  update      atualiza a família (ou um app)");
    println!("  sys         utilitários de sistema");
    println!("  version     versão do opt");
    println!("  help        esta ajuda");
    println!();
    println!("  use    opt <app> [args...]    para rodar um app");
    println!("  alias  {}", alias_line());
}

/// Full routing table: id | bins | cargo | AUR | about.
/// Plain B&W text (no ANSI here); mirrors README "Routing & packages".
fn print_routing_table() {
    println!(
        "  {:<9} {:<22} {:<16} {:<16} {}",
        "app", "bins", "cargo", "aur", "about"
    );
    for spec in all() {
        println!(
            "  {:<9} {:<22} {:<16} {:<16} {}",
            spec.id,
            spec.bins.join(", "),
            spec.cargo,
            spec.aur,
            spec.about
        );
    }
}

fn alias_line() -> String {
    aliases()
        .iter()
        .map(|(a, t)| format!("{a} → {t}"))
        .collect::<Vec<_>>()
        .join("   ")
}

fn print_help() {
    println!("opt — the Option family CLI");
    println!();
    println!("USAGE:");
    println!("    opt                    list family apps");
    println!("    opt <app> [args...]    run an app, forwarding arguments");
    println!("    opt status             show installed apps + versions");
    println!("    opt doctor             check each app's system dependencies");
    println!("    opt install [app...]   install the family (or specific apps)");
    println!("    opt update  [app...]   update the family (or specific apps)");
    println!("    opt sys <util>         system utilities (see opt sys --help)");
    println!("    opt version            print the opt version");
    println!("    opt help               print this help");
    println!();
    println!("APPS (routing & packages):");
    println!(
        "    {:<9} {:<22} {:<16} {:<16} {}",
        "app", "bins", "cargo", "aur", "about"
    );
    for spec in all() {
        println!(
            "    {:<9} {:<22} {:<16} {:<16} {}",
            spec.id,
            spec.bins.join(", "),
            spec.cargo,
            spec.aur,
            spec.about
        );
    }
    println!();
    println!("ALIASES:");
    println!("    {}", alias_line());
    println!();
    println!("ENVIRONMENT:");
    println!("    OPTION_BIN_<ID>    force an app binary path (e.g. OPTION_BIN_MUSIC)");
    println!("    OPTION_PKG            package manager: cargo (default) | yay | paru | pacman");
    println!();
    println!("FAMILY METAPACKAGE (Arch):");
    println!("    opt install family   yay -S option-family   (ou paru/pacman)");
}
