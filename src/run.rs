//! Forward the remaining arguments to a resolved app binary, preserving
//! stdio and the child's exit status.

use std::path::PathBuf;
use std::process::{Command, ExitCode, Stdio};

/// Run `bin` with `args`, inheriting stdio, and mirror its exit status.
///
/// If the child is terminated by a signal (no conventional exit code), the
/// shell convention `128 + signal` is used.
pub fn run(bin: &PathBuf, args: &[String]) -> Result<ExitCode, String> {
    let status = Command::new(bin)
        .args(args)
        .stdin(Stdio::inherit())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .status()
        .map_err(|error| format!("cannot run {}: {error}", bin.display()))?;

    Ok(match status.code() {
        Some(code) => ExitCode::from(code as u8),
        None => ExitCode::from((128 + signal_number(&status)) as u8),
    })
}

/// Best-effort signal number from a `std::process::ExitStatus`.
fn signal_number(status: &std::process::ExitStatus) -> u8 {
    #[cfg(unix)]
    {
        use std::os::unix::process::ExitStatusExt;
        if let Some(signal) = status.signal() {
            return signal as u8;
        }
    }
    0
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn echoes_args_through() {
        // `sh -c` prints each positional arg; verify passthrough of spaces
        // and flags without the hub interpreting them.
        let sh = PathBuf::from("/bin/sh");
        let args = vec![
            "-c".to_string(),
            r#"printf '%s\n' "$@" # helper"#.to_string(),
            "helper".to_string(),
            "some file with spaces".to_string(),
            "--flag".to_string(),
        ];
        let code = run(&sh, &args).unwrap();
        assert_eq!(code, ExitCode::SUCCESS);
    }
}
