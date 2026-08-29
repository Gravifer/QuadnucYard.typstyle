mod common;

use common::{Workspace, typstyle_cmd_snapshot};
use insta_cmd::{Spawn, SpawnExt};

const STDIN: &str = "#let  x  = (1+2)";

#[test]
fn test_nothing() {
    let space = Workspace::new();

    typstyle_cmd_snapshot!(space.cli(), @r"
    success: true
    exit_code: 0
    ----- stdout -----


    ----- stderr -----
    ");
}

#[test]
fn test_stdin() {
    let space = Workspace::new();

    typstyle_cmd_snapshot!(space.cli().pass_stdin(STDIN), @r"
    success: true
    exit_code: 0
    ----- stdout -----
    #let x = (1 + 2)

    ----- stderr -----
    ");
}

#[test]
fn test_default_lf_stdin_normalizes_crlf() {
    let space = Workspace::new();
    let mut command = space.cli();
    let (_, output) = command
        .pass_stdin(b"#let value  =  1\r\n")
        .spawn_with_info(None);

    assert!(output.status.success());
    assert_eq!(output.stdout, b"#let value = 1\n");
    assert!(output.stderr.is_empty());
}

#[test]
fn test_crlf_preserve_stdin_uses_crlf() {
    let space = Workspace::new();
    let mut command = space.cli();
    command.arg("--line-ending=crlf-preserve");
    let (_, output) = command
        .pass_stdin(b"#let value  =  1\r\n")
        .spawn_with_info(None);

    assert!(output.status.success());
    assert_eq!(output.stdout, b"#let value = 1\r\n");
    assert!(output.stderr.is_empty());
}

#[test]
fn test_first_line_structural_stdin_only_converts_trivia() {
    let space = Workspace::new();
    let mut command = space.cli();
    command.arg("--line-ending=first-line-structural");
    let (_, output) = command
        .pass_stdin(b"#let string  =  \"a\r\nb\"\r\n#let value  =  1\r\n")
        .spawn_with_info(None);

    assert!(output.status.success());
    assert_eq!(
        output.stdout,
        b"#let string = \"a\nb\"\r\n#let value = 1\r\n"
    );
    assert!(output.stderr.is_empty());
}

#[test]
fn test_crlf_preserve_stdin_check_unchanged() {
    let space = Workspace::new();

    typstyle_cmd_snapshot!(space
        .cli()
        .args(["--check", "--line-ending=crlf-preserve"])
        .pass_stdin(b"#let value = 1\r\n"), @r"
    success: true
    exit_code: 0
    ----- stdout -----

    ----- stderr -----
    ");
}

#[test]
fn test_stdin_erroneous() {
    let space = Workspace::new();

    typstyle_cmd_snapshot!(space.cli().pass_stdin("#"), @r"
    success: true
    exit_code: 0
    ----- stdout -----
    #
    ----- stderr -----
    warn: Failed to parse stdin. The source is erroneous.
    ");
}

#[test]
fn test_stdin_column() {
    let space = Workspace::new();

    typstyle_cmd_snapshot!(space.cli().args(["-c=0"]).pass_stdin(STDIN), @r"
    success: true
    exit_code: 0
    ----- stdout -----
    #let x = (
      1
        + 2
    )

    ----- stderr -----
    ");
}

#[test]
fn test_stdin_check() {
    let space = Workspace::new();

    typstyle_cmd_snapshot!(space.cli().args(["--check"]).pass_stdin(STDIN), @r"
    success: false
    exit_code: 1
    ----- stdout -----

    ----- stderr -----
    ");
}

#[test]
fn test_stdin_diff() {
    let space: Workspace = Workspace::new();

    typstyle_cmd_snapshot!(space.cli().args(["--diff"]).pass_stdin(STDIN), @r"
    success: false
    exit_code: 1
    ----- stdout -----
    @@ -1 +1 @@
    -#let  x  = (1+2)
    \ No newline at end of file
    +#let x = (1 + 2)


    ----- stderr -----
    ");
}

#[test]
fn test_stdin_inplace() {
    let space = Workspace::new();

    typstyle_cmd_snapshot!(space.cli().args(["-i"]).pass_stdin(STDIN), @r"
    success: false
    exit_code: 2
    ----- stdout -----

    ----- stderr -----
    error: cannot perform in-place formatting without at least one file being presented

    Usage: typstyle [OPTIONS] [INPUT]...

    For more information, try '--help'.
    ");
}

#[test]
fn test_stdin_inplace_check() {
    let space = Workspace::new();

    typstyle_cmd_snapshot!(space.cli().args(["-i", "--check"]).pass_stdin(STDIN), @r"
    success: false
    exit_code: 2
    ----- stdout -----

    ----- stderr -----
    error: the argument '--inplace' cannot be used with '--check'

    Usage: typstyle --inplace [INPUT]...

    For more information, try '--help'.
    ");
}

#[test]
fn test_stdin_debug_ast() {
    let space = Workspace::new();

    typstyle_cmd_snapshot!(space.cli().args(["-a"]).pass_stdin(STDIN), @r##"
    success: true
    exit_code: 0
    ----- stdout -----
    Markup: 16 [
      Hash: "#",
      LetBinding: 15 [
        Let: "let",
        Space: "  ",
        Ident: "x",
        Space: "  ",
        Eq: "=",
        Space: " ",
        Parenthesized: 5 [
          LeftParen: "(",
          Binary: 3 [
            Int: "1",
            Plus: "+",
            Int: "2",
          ],
          RightParen: ")",
        ],
      ],
    ]
    #let x = (1 + 2)

    ----- stderr -----
    "##);
}
