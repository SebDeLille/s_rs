//! Loading of user-provided Scheme startup scripts.
//!
//! On launch, front-ends (the `srs` CLI, `srsgtk`, ...) can call
//! [`load_startup_scripts`] to evaluate every `*.scm` file found under
//! the `startup/` subdirectory of:
//!
//! 1. `$HOME/.config/srs/startup/` (global scripts, shared across launch
//!    directories), in alphabetical order;
//! 2. `.srs/startup/` under the current working directory (the directory
//!    the binary was launched from), in alphabetical order.
//!
//! The sibling `libs/` directory, e.g. `$HOME/.config/srs/libs/`, is
//! reserved for libraries loaded on demand via `(load ...)` and is not
//! scanned automatically. This lets users extend the interpreter with
//! new bindings (procedures, macros via `define`, ...) without
//! recompiling. A missing directory is silently skipped. A script that
//! fails to load (I/O error, syntax error, or evaluation error) reports
//! its error on stderr and does not prevent the remaining scripts from
//! being loaded.

use std::path::{Path, PathBuf};
use std::rc::Rc;

use crate::interpretor::repl::{EvalOutcome, eval_source};
use crate::types::core::Env;

/// Evaluates every `*.scm` startup script found in the global config
/// directory (`$HOME/.config/srs/startup`) and then in the current
/// working directory (`.srs/startup`), in that order, binding new
/// definitions into `env`.
///
/// Errors are reported on stderr and do not stop the loading of the
/// remaining scripts.
pub fn load_startup_scripts(env: &Rc<Env>) {
    for dir in startup_dirs() {
        for path in scm_files_in(&dir) {
            if let Err(message) = load_script(&path, env) {
                eprintln!("erreur: startup: {}: {}", path.display(), message);
            }
        }
    }
}

/// Directories scanned for startup scripts, in load order: the global
/// config directory first (`$HOME/.config/srs/startup`), then `.srs/startup`
/// under the current working directory. Either (or both) may be absent if
/// `$HOME` is unset or the cwd cannot be determined, in which case they are
/// simply omitted.
fn startup_dirs() -> Vec<PathBuf> {
    let mut dirs = Vec::with_capacity(2);

    if let Ok(home) = std::env::var("HOME")
        && !home.is_empty()
    {
        dirs.push(
            PathBuf::from(home)
                .join(".config")
                .join("srs")
                .join("startup"),
        );
    }

    if let Ok(cwd) = std::env::current_dir() {
        dirs.push(cwd.join(".srs").join("startup"));
    }

    dirs
}

/// Lists the `*.scm` regular files directly inside `dir`, sorted
/// alphabetically by file name. Returns an empty list if `dir` doesn't
/// exist or can't be read.
fn scm_files_in(dir: &Path) -> Vec<PathBuf> {
    let entries = match std::fs::read_dir(dir) {
        Ok(entries) => entries,
        Err(_) => return Vec::new(),
    };

    let mut files: Vec<PathBuf> = entries
        .filter_map(|entry| entry.ok())
        .map(|entry| entry.path())
        .filter(|path| {
            path.is_file() && path.extension().and_then(|ext| ext.to_str()) == Some("scm")
        })
        .collect();

    files.sort();
    files
}

/// Reads and evaluates a single startup script in `env`.
fn load_script(path: &Path, env: &Rc<Env>) -> Result<(), String> {
    let source = std::fs::read_to_string(path).map_err(|e| e.to_string())?;

    match eval_source(&source, env)? {
        EvalOutcome::Done(_) => Ok(()),
        EvalOutcome::Incomplete => Err("unexpected end of input".to_string()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::interpretor::evaluator::global_env;
    use crate::types::core::SrsValue;

    fn write_script(dir: &Path, name: &str, content: &str) {
        std::fs::write(dir.join(name), content).unwrap();
    }

    #[test]
    fn startup_dirs_point_to_startup_subdirectories() {
        let dirs = startup_dirs();
        assert_eq!(dirs.len(), 2);
        assert!(
            dirs[0].ends_with(".config/srs/startup"),
            "global dir: {:?}",
            dirs[0]
        );
        assert!(
            dirs[1].ends_with(".srs/startup"),
            "local dir: {:?}",
            dirs[1]
        );
    }

    #[test]
    fn startup_dirs_omits_global_when_home_is_unset() {
        let original = std::env::var_os("HOME");
        unsafe {
            std::env::remove_var("HOME");
        }
        let dirs = startup_dirs();
        assert_eq!(dirs.len(), 1);
        assert!(dirs[0].ends_with(".srs/startup"));
        if let Some(home) = original {
            unsafe {
                std::env::set_var("HOME", home);
            }
        }
    }

    #[test]
    fn loads_scripts_in_alphabetical_order_and_binds_definitions() {
        let dir = tempdir();
        write_script(&dir, "b.scm", "(define from-b 2)");
        write_script(&dir, "a.scm", "(define from-a 1)");
        write_script(&dir, "not-scheme.txt", "(define ignored 99)");

        let env = global_env();
        for path in scm_files_in(&dir) {
            load_script(&path, &env).unwrap();
        }

        assert!(matches!(env.get("from-a"), Some(SrsValue::Integer(1))));
        assert!(matches!(env.get("from-b"), Some(SrsValue::Integer(2))));
        assert!(env.get("ignored").is_none());
    }

    #[test]
    fn missing_directory_yields_no_files() {
        let dir = tempdir().join("does-not-exist");
        assert!(scm_files_in(&dir).is_empty());
    }

    #[test]
    fn error_in_one_script_does_not_prevent_others_from_loading() {
        let dir = tempdir();
        write_script(&dir, "1-bad.scm", "(this-is-not-bound)");
        write_script(&dir, "2-good.scm", "(define loaded-ok #t)");

        let env = global_env();
        for path in scm_files_in(&dir) {
            let _ = load_script(&path, &env);
        }

        assert!(matches!(
            env.get("loaded-ok"),
            Some(SrsValue::Boolean(true))
        ));
    }

    /// Creates a fresh temporary directory for a test, returning its path.
    fn tempdir() -> PathBuf {
        let mut dir = std::env::temp_dir();
        let unique = format!(
            "srs-startup-test-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        );
        dir.push(unique);
        std::fs::create_dir_all(&dir).unwrap();
        dir
    }
}
