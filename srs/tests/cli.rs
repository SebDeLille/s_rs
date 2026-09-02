use std::io::Write;
use std::process::{Command, Stdio};

fn bin() -> &'static str {
    env!("CARGO_BIN_EXE_srs")
}

#[test]
fn one_shot_addition() {
    let output = Command::new(bin())
        .arg("(+ 2 3)")
        .output()
        .expect("failed to run binary");

    assert!(output.status.success(), "unexpected exit: {:?}", output);
    assert_eq!("= 5\n", String::from_utf8_lossy(&output.stdout));
    assert!(output.stderr.is_empty());
}

#[test]
fn one_shot_exit_code() {
    let output = Command::new(bin())
        .arg("(exit 5)")
        .output()
        .expect("failed to run binary");

    assert_eq!(Some(5), output.status.code());
    assert!(output.stdout.is_empty());
    assert!(output.stderr.is_empty());
}

#[test]
fn one_shot_division_by_zero_exits_with_error() {
    let output = Command::new(bin())
        .arg("(/ 1 0)")
        .output()
        .expect("failed to run binary");

    assert_eq!(Some(1), output.status.code());
    assert!(output.stdout.is_empty());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("error"), "stderr: {}", stderr);
}

#[test]
fn repl_evaluates_piped_input() {
    let mut child = Command::new(bin())
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("failed to spawn repl");

    {
        let stdin = child.stdin.as_mut().expect("stdin not captured");
        stdin.write_all(b"(+ 2 3)\n(exit)\n").expect("write failed");
    }

    let output = child.wait_with_output().expect("failed to wait");
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("= 5"), "stdout: {}", stdout);
    assert!(output.stderr.is_empty());
}

#[test]
fn repl_quits_on_exit_command() {
    let mut child = Command::new(bin())
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("failed to spawn repl");

    {
        let stdin = child.stdin.as_mut().expect("stdin not captured");
        stdin.write_all(b"exit\n").expect("write failed");
    }

    let output = child.wait_with_output().expect("failed to wait");
    assert!(output.status.success());
    assert!(String::from_utf8_lossy(&output.stdout).is_empty());
    assert!(output.stderr.is_empty());
}

#[test]
fn repl_continues_after_error() {
    let mut child = Command::new(bin())
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("failed to spawn repl");

    {
        let stdin = child.stdin.as_mut().expect("stdin not captured");
        stdin
            .write_all(b"(foo)\n(+ 1 2)\n(exit)\n")
            .expect("write failed");
    }

    let output = child.wait_with_output().expect("failed to wait");
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stdout.contains("= 3"), "stdout: {}", stdout);
    assert!(stderr.contains("error"), "stderr: {}", stderr);
}
