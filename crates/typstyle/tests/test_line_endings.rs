mod common;

use std::{fs, path::Path, process::Output};

use common::{Workspace, typstyle_cmd_snapshot};
use insta_cmd::{Spawn, SpawnExt};

fn run_stdin(args: &[&str], input: &[u8]) -> Output {
    let space = Workspace::new();
    let mut command = space.cli();
    command.args(args);
    let (_, output) = command.pass_stdin(input).spawn_with_info(None);
    output
}

fn read_bytes(space: &Workspace, path: impl AsRef<Path>) -> Vec<u8> {
    fs::read(space.project_path().join(path)).unwrap()
}

// CLI contract

#[test]
fn test_line_ending_help_lists_modes_and_default() {
    let space = Workspace::new();
    let output = space.cli().arg("--help").output().unwrap();
    let stdout = String::from_utf8(output.stdout).unwrap();

    assert!(output.status.success());
    assert!(stdout.contains("--line-ending <LINE_ENDING>"));
    assert!(stdout.contains("[default: lf]"));
    assert!(stdout.contains("Possible values:"));
    assert!(stdout.contains("- lf:"));
    assert!(stdout.contains("- crlf-preserve:"));
}

#[test]
fn test_line_ending_rejects_unknown_mode() {
    let space = Workspace::new();

    typstyle_cmd_snapshot!(space.cli().args(["--line-ending=auto"]), @r"
    success: false
    exit_code: 2
    ----- stdout -----

    ----- stderr -----
    error: invalid value 'auto' for '--line-ending <LINE_ENDING>'
      [possible values: lf, crlf-preserve]

    For more information, try '--help'.
    ");
}

// Default and explicit LF behavior

#[test]
fn test_default_lf_check_rejects_crlf_file() {
    let mut space = Workspace::new();
    space.write_tracked("a.typ", b"#let a = 0\r\n");

    typstyle_cmd_snapshot!(space.cli().args(["a.typ", "--check"]), @r"
    success: false
    exit_code: 1
    ----- stdout -----
    Would reformat: a.typ

    ----- stderr -----
    ");

    assert!(space.all_unmodified());
}

#[test]
fn test_default_lf_normalizes_file_output() {
    let mut space = Workspace::new();
    space.write_tracked("a.typ", b"#let a  =  0\r\n");

    let output = space.cli().arg("a.typ").output().unwrap();

    assert!(output.status.success());
    assert_eq!(output.stdout, b"#let a = 0\n");
    assert!(output.stderr.is_empty());
    assert!(space.all_unmodified());
}

#[test]
fn test_default_lf_normalizes_stdin_output() {
    let output = run_stdin(&[], b"#let value  =  1\r\n");

    assert!(output.status.success());
    assert_eq!(output.stdout, b"#let value = 1\n");
    assert!(output.stderr.is_empty());
}

#[test]
fn test_explicit_lf_normalizes_file_inplace() {
    let mut space = Workspace::new();
    space.write_tracked("a.typ", b"#let a  =  0\r\n#let b  =  1");

    typstyle_cmd_snapshot!(space.cli().args(["a.typ", "--inplace", "--line-ending=lf"]), @r"
    success: true
    exit_code: 0
    ----- stdout -----

    ----- stderr -----
    ");

    assert_eq!(read_bytes(&space, "a.typ"), b"#let a = 0\n#let b = 1\n");
}

// CRLF-preserve policy

#[test]
fn test_crlf_preserve_selection_matrix() {
    let cases = [
        (
            "consistent CRLF",
            "#let a  =  0\r\n#let b  =  1\r\n",
            "#let a = 0\r\n#let b = 1\r\n",
        ),
        (
            "LF",
            "#let a  =  0\n#let b  =  1\n",
            "#let a = 0\n#let b = 1\n",
        ),
        ("no newline", "#let a  =  0", "#let a = 0\n"),
        (
            "CRLF then LF",
            "#let a  =  0\r\n#let b  =  1\n",
            "#let a = 0\n#let b = 1\n",
        ),
        (
            "LF then CRLF",
            "#let a  =  0\n#let b  =  1\r\n",
            "#let a = 0\n#let b = 1\n",
        ),
        (
            "bare CR",
            "#let value  =  \"a\rb\"",
            "#let value = \"a\rb\"\n",
        ),
        (
            "CRLF and bare CR",
            "#let a  =  0\r\n#let b  =  1\r#let c  =  2",
            "#let a = 0\n#let b = 1\r#let c = 2\n",
        ),
        (
            "non-ASCII separators",
            "#let value  =  \"a\u{000b}b\u{000c}c\u{0085}d\u{2028}e\u{2029}f\"\r\n",
            "#let value = \"a\u{000b}b\u{000c}c\u{0085}d\u{2028}e\u{2029}f\"\r\n",
        ),
    ];

    for (name, input, expected) in cases {
        let output = run_stdin(&["--line-ending=crlf-preserve"], input.as_bytes());

        assert!(output.status.success(), "{name}: {output:?}");
        assert_eq!(output.stdout, expected.as_bytes(), "{name}");
        assert!(output.stderr.is_empty(), "{name}: {output:?}");
    }
}

#[test]
fn test_crlf_preserve_file_output_uses_crlf() {
    let mut space = Workspace::new();
    space.write_tracked("a.typ", b"#let a  =  0\r\n");

    let output = space
        .cli()
        .args(["a.typ", "--line-ending=crlf-preserve"])
        .output()
        .unwrap();

    assert!(output.status.success());
    assert_eq!(output.stdout, b"#let a = 0\r\n");
    assert!(output.stderr.is_empty());
    assert!(space.all_unmodified());
}

#[test]
fn test_crlf_preserve_check_accepts_formatted_file() {
    let mut space = Workspace::new();
    space.write_tracked("a.typ", b"#let a = 0\r\n");

    typstyle_cmd_snapshot!(space.cli().args([
        "a.typ",
        "--check",
        "--line-ending=crlf-preserve",
    ]), @r"
    success: true
    exit_code: 0
    ----- stdout -----

    ----- stderr -----
    ");

    assert!(space.all_unmodified());
}

#[test]
fn test_crlf_preserve_check_reports_unformatted_file() {
    let mut space = Workspace::new();
    space.write_tracked("a.typ", b"#let a  =  0\r\n");

    typstyle_cmd_snapshot!(space.cli().args([
        "a.typ",
        "--check",
        "--line-ending=crlf-preserve",
    ]), @r"
    success: false
    exit_code: 1
    ----- stdout -----
    Would reformat: a.typ

    ----- stderr -----
    ");

    assert!(space.all_unmodified());
}

#[test]
fn test_crlf_preserve_diff_accepts_formatted_file() {
    let mut space = Workspace::new();
    space.write_tracked("a.typ", b"#let a = 0\r\n");

    typstyle_cmd_snapshot!(space.cli().args([
        "a.typ",
        "--diff",
        "--line-ending=crlf-preserve",
    ]), @r"
    success: true
    exit_code: 0
    ----- stdout -----

    ----- stderr -----
    ");

    assert!(space.all_unmodified());
}

#[test]
fn test_crlf_preserve_diff_keeps_control_lines_lf() {
    let output = run_stdin(
        &["--diff", "--line-ending=crlf-preserve"],
        b"#let value  =  1\r\n",
    );

    assert_eq!(output.status.code(), Some(1));
    assert_eq!(
        output.stdout,
        b"@@ -1 +1 @@\n-#let value  =  1\r\n+#let value = 1\r\n\n"
    );
    assert!(output.stderr.is_empty());
}

#[test]
fn test_crlf_preserve_inplace_converges() {
    let mut space = Workspace::new();
    space.write_tracked("a.typ", b"#let a  =  0\r\n#let b  =  1");

    typstyle_cmd_snapshot!(space.cli().args([
        "a.typ",
        "--inplace",
        "--line-ending=crlf-preserve",
    ]), @r"
    success: true
    exit_code: 0
    ----- stdout -----

    ----- stderr -----
    ");

    assert_eq!(read_bytes(&space, "a.typ"), b"#let a = 0\r\n#let b = 1\r\n");

    let check = space
        .cli()
        .args(["a.typ", "--check", "--line-ending=crlf-preserve"])
        .output()
        .unwrap();
    assert!(check.status.success());
    assert!(check.stdout.is_empty());
    assert!(check.stderr.is_empty());
}

#[test]
fn test_crlf_preserve_multiline_payload_is_idempotent() {
    let input = b"#let string  =  \"a\r\nb\"\r\n#let raw  =  ```a\r\nb```\r\n";
    let first = run_stdin(&["--line-ending=crlf-preserve"], input);

    assert!(first.status.success());
    assert_eq!(
        first.stdout,
        b"#let string = \"a\r\nb\"\r\n#let raw = ```a\r\nb```\r\n"
    );
    assert!(first.stderr.is_empty());

    let second = run_stdin(&["--line-ending=crlf-preserve"], &first.stdout);
    assert!(second.status.success());
    assert_eq!(second.stdout, first.stdout);
    assert!(second.stderr.is_empty());

    let check = run_stdin(&["--check", "--line-ending=crlf-preserve"], &first.stdout);
    assert!(check.status.success());
    assert!(check.stdout.is_empty());
    assert!(check.stderr.is_empty());
}
