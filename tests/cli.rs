//! Integration tests for the `lino-transformer` command-line interface
//! (requirement R15). They invoke the compiled binary and assert on stdout,
//! stderr, and exit codes.

use std::io::Write;
use std::process::{Command, Stdio};

/// Runs the built binary with `args`, feeding `stdin`, and returns
/// `(stdout, stderr, exit_code)`.
fn run_cli(args: &[&str], stdin: &str) -> (String, String, i32) {
    let binary = env!("CARGO_BIN_EXE_lino-transformer");
    let mut child = Command::new(binary)
        .args(args)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("spawn lino-transformer");
    child
        .stdin
        .take()
        .expect("stdin")
        .write_all(stdin.as_bytes())
        .expect("write stdin");
    let output = child.wait_with_output().expect("wait");
    (
        String::from_utf8_lossy(&output.stdout).into_owned(),
        String::from_utf8_lossy(&output.stderr).into_owned(),
        output.status.code().unwrap_or(-1),
    )
}

fn example_path(name: &str) -> String {
    format!("{}/examples/{}", env!("CARGO_MANIFEST_DIR"), name)
}

#[test]
fn run_swap_prints_result_and_succeeds() {
    let (stdout, stderr, code) = run_cli(
        &["run", "--program", &example_path("swap.lino")],
        "(a before b)",
    );
    assert_eq!(stdout.trim_end(), "(b before a)");
    assert!(stderr.contains("normal form"));
    assert_eq!(code, 0);
}

#[test]
fn check_reports_rule_count() {
    let (stdout, _stderr, code) =
        run_cli(&["check", "--program", &example_path("turing-bb2.lino")], "");
    assert!(stdout.contains("8 rules"));
    assert_eq!(code, 0);
}

#[test]
fn bounded_stop_uses_exit_code_three() {
    let (_stdout, stderr, code) = run_cli(
        &[
            "run",
            "--program",
            &example_path("turing-runaway.lino"),
            "--max-steps",
            "3",
        ],
        "(tm e R 0 e)",
    );
    assert!(stderr.contains("step limit"));
    assert_eq!(code, 3);
}

#[test]
fn compile_error_uses_exit_code_two() {
    // A program that references an unbound template variable must not compile.
    let dir = std::env::temp_dir();
    let path = dir.join("lino_cli_bad_program.lino");
    std::fs::write(&path, "(rule: (match: ($x)) (replace: ($y)))").unwrap();
    let (_stdout, stderr, code) =
        run_cli(&["run", "--program", path.to_str().unwrap()], "(a)");
    assert!(stderr.contains("did not compile"));
    assert_eq!(code, 2);
}

#[test]
fn step_applies_a_single_rule() {
    let (stdout, stderr, code) = run_cli(
        &["step", "--program", &example_path("peano-add.lino")],
        "(add (s z) (s z))",
    );
    // One step rewrites the outermost add via add-succ.
    assert_eq!(stdout.trim_end(), "(s (add z (s z)))");
    assert!(stderr.contains("applied: add-succ"));
    assert_eq!(code, 0);
}

#[test]
fn help_and_version_short_circuit() {
    let (stdout, _stderr, code) = run_cli(&["--help"], "");
    assert!(stdout.contains("USAGE"));
    assert_eq!(code, 0);

    let (stdout, _stderr, code) = run_cli(&["--version"], "");
    assert!(stdout.contains("lino-transformer"));
    assert_eq!(code, 0);
}
