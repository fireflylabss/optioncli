//! opt — the Option family CLI.
//!
//! `opt <app> [args...]` forwards to the matching app binary. It also offers
//! family-level commands: `gui`, `status`, `doctor`, `install`, `sys` and
//! `version`. It does not implement any app logic itself; it only knows how
//! to find, install and check each app.

mod apps;
mod doctor;
mod install;
mod run;
mod status;
mod sys;

use std::env;
use std::path::Path;
use std::process::ExitCode;

use apps::{
    AppSpec, aliases, all, desktop_bins, find_binary, find_desktop_binary, find_gui_binary,
    gui_install_hint, has_desktop_surface, install_hint, lookup,
};
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
        // `gui` is a reserved verb (like status/doctor): no app id uses it.
        "gui" => gui(&args[1..]),
        app_id => dispatch(app_id, &args[1..]),
    }
}

/// Run `bin` with `args`, reporting a spawn failure as an `opt:` error.
fn run_binary(bin: &Path, args: &[String]) -> ExitCode {
    match run(bin, args) {
        Ok(code) => code,
        Err(error) => {
            eprintln!("opt: {error}");
            ExitCode::FAILURE
        }
    }
}

/// Resolve `app_id`, printing the known ids when it is not a family app.
fn resolve(app_id: &str) -> Option<&'static AppSpec> {
    let spec = lookup(app_id);
    if spec.is_none() {
        eprintln!("opt: '{app_id}' não é um app Option conhecido.");
        eprintln!("Apps: {}", known_ids());
    }
    spec
}

/// Run the app matching `app_id`, forwarding the rest of the arguments.
fn dispatch(app_id: &str, rest: &[String]) -> ExitCode {
    let Some(spec) = resolve(app_id) else {
        return ExitCode::FAILURE;
    };

    match find_binary(spec) {
        Some(bin) => run_binary(&bin, rest),
        None => {
            eprintln!("opt: '{}' não encontrado.", spec.id);
            eprintln!("       Instale com: {}", install_hint(spec));
            ExitCode::FAILURE
        }
    }
}

/// `opt gui <app> [args...]` — run the app's desktop front-end.
///
/// Bare `opt gui` lists the configured front-ends and their state. Apps that
/// are already desktop apps (`is_gui`, e.g. optionTerm) have no separate
/// front-end, so `opt gui <app>` just runs their only binary.
fn gui(args: &[String]) -> ExitCode {
    let Some(app_id) = args.first() else {
        gui_list();
        return ExitCode::SUCCESS;
    };
    let Some(spec) = resolve(app_id) else {
        return ExitCode::FAILURE;
    };
    let rest = &args[1..];

    // optionTerm & friends: the app itself is the desktop surface.
    if spec.gui_bins.is_empty() && spec.is_gui {
        return dispatch(spec.id, rest);
    }

    match find_gui_binary(spec) {
        Some(bin) => run_binary(&bin, rest),
        None => {
            eprintln!("opt: '{}' não tem interface desktop instalada.", spec.id);
            if spec.gui_bins.is_empty() {
                eprintln!("       Este app não tem front-end desktop separado.");
            } else {
                eprintln!("       Procurado: {}", spec.gui_bins.join(", "));
            }
            if let Some(hint) = gui_install_hint(spec) {
                eprintln!("       Instale com: {hint}");
            }
            ExitCode::FAILURE
        }
    }
}

/// List every app `opt gui` can open, and its state. Apps that are already
/// desktop apps are listed under their own binary.
fn gui_list() {
    for spec in all().iter().filter(|s| has_desktop_surface(s)) {
        let state = match find_desktop_binary(spec) {
            Some(bin) => bin
                .file_name()
                .map(|name| name.to_string_lossy().into_owned())
                .unwrap_or_else(|| bin.display().to_string()),
            None => format!("não instalado ({})", desktop_bins(spec).join(", ")),
        };
        println!("  {:<9} {}", spec.id, state);
    }
}

fn known_ids() -> String {
    all().iter().map(|s| s.id).collect::<Vec<_>>().join(", ")
}

fn print_menu() {
    println!();
    println!("{MENU_HEADER}");
    println!();
    print_routing_table("  ");
    println!();
    println!("  status      apps instalados + versões");
    println!("  doctor      dependências de sistema de cada app");
    println!("  install     instala a família (ou um app)");
    println!("  update      atualiza a família (ou um app)");
    println!("  gui         interface desktop de um app");
    println!("  sys         utilitários de sistema");
    println!("  version     versão do opt");
    println!("  help        esta ajuda");
    println!();
    println!("  use      opt <app> [args...]    para rodar um app");
    println!("  desktop  {}", gui_line());
    println!("  alias    {}", alias_line());
}

/// Full routing table: id | bins | cargo | AUR | about, printed with
/// `indent` leading spaces. Column widths follow the longest cell, so a
/// long bin list (e.g. search's) never pushes the row out of alignment.
/// Plain B&W text (no ANSI here); mirrors README "Routing & packages".
fn print_routing_table(indent: &str) {
    let bins: Vec<String> = all().iter().map(|s| s.bins.join(", ")).collect();
    let width = |header: &str, cells: &mut dyn Iterator<Item = &str>| {
        cells.map(str::len).chain([header.len()]).max().unwrap_or(0)
    };
    let w_app = width("app", &mut all().iter().map(|s| s.id));
    let w_bins = width("bins", &mut bins.iter().map(String::as_str));
    let w_cargo = width("cargo", &mut all().iter().map(|s| s.cargo));
    let w_aur = width("aur", &mut all().iter().map(|s| s.aur));

    println!(
        "{indent}{:<w_app$} {:<w_bins$} {:<w_cargo$} {:<w_aur$} about",
        "app", "bins", "cargo", "aur"
    );
    for (spec, bins) in all().iter().zip(&bins) {
        println!(
            "{indent}{:<w_app$} {:<w_bins$} {:<w_cargo$} {:<w_aur$} {}",
            spec.id, bins, spec.cargo, spec.aur, spec.about
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

/// `app → desktop bins` pairs for the menu / help (apps without any desktop
/// surface omitted).
fn gui_line() -> String {
    all()
        .iter()
        .filter(|s| has_desktop_surface(s))
        .map(|s| format!("{} → {}", s.id, desktop_bins(s).join(", ")))
        .collect::<Vec<_>>()
        .join("   ")
}

fn print_help() {
    println!("opt — the Option family CLI");
    println!();
    println!("USAGE:");
    println!("    opt                    list family apps");
    println!("    opt <app> [args...]    run an app, forwarding arguments");
    println!("    opt gui <app> [...]    run an app's desktop front-end");
    println!("    opt status             show installed apps + versions");
    println!("    opt doctor             check each app's system dependencies");
    println!("    opt install [app...]   install the family (or specific apps)");
    println!("    opt update  [app...]   update the family (or specific apps)");
    println!("    opt sys <util>         system utilities (see opt sys --help)");
    println!("    opt version            print the opt version");
    println!("    opt help               print this help");
    println!();
    println!("APPS (routing & packages):");
    print_routing_table("    ");
    println!();
    println!("GUIS (opt gui <app>):");
    println!("    {}", gui_line());
    println!();
    println!("ALIASES:");
    println!("    {}", alias_line());
    println!();
    println!("ENVIRONMENT:");
    println!(
        "    {:<22} force an app binary path (e.g. OPTION_BIN_MUSIC)",
        "OPTION_BIN_<ID>"
    );
    println!(
        "    {:<22} force a GUI binary path (e.g. OPTION_GUI_BIN_FILES)",
        "OPTION_GUI_BIN_<ID>"
    );
    println!(
        "    {:<22} package manager: cargo (default) | yay | paru | pacman",
        "OPTION_PKG"
    );
    println!();
    println!("FAMILY METAPACKAGE (Arch):");
    println!("    opt install family   yay -S option-family   (ou paru/pacman)");
}
