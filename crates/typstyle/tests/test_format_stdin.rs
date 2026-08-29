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
fn test_crlf_preserve_stdin_is_idempotent() {
    let space = Workspace::new();
    let input = b"#let string  =  \"a\r\nb\"\r\n#let raw  =  ```a\r\nb```\r\n";

    let mut first_command = space.cli();
    first_command.arg("--line-ending=crlf-preserve");
    let (_, first) = first_command.pass_stdin(input).spawn_with_info(None);

    let mut second_command = space.cli();
    second_command.arg("--line-ending=crlf-preserve");
    let (_, second) = second_command
        .pass_stdin(first.stdout.clone())
        .spawn_with_info(None);

    let mut check_command = space.cli();
    check_command.args(["--check", "--line-ending=crlf-preserve"]);
    let (_, check) = check_command
        .pass_stdin(first.stdout.clone())
        .spawn_with_info(None);

    assert!(first.status.success());
    assert!(first.stderr.is_empty());
    assert_eq!(first.stdout, second.stdout);
    assert!(second.status.success());
    assert!(second.stderr.is_empty());
    assert!(check.status.success());
    assert!(check.stdout.is_empty());
    assert!(check.stderr.is_empty());
}

#[test]
fn test_crlf_preserve_stdin_without_line_endings_uses_formatter_output() {
    let space = Workspace::new();
    let mut command = space.cli();
    command.arg("--line-ending=crlf-preserve");
    let (_, output) = command
        .pass_stdin(b"#let value  =  1")
        .spawn_with_info(None);

    assert!(output.status.success());
    assert_eq!(output.stdout, b"#let value = 1\n");
    assert!(output.stderr.is_empty());
}

#[test]
fn test_crlf_preserve_stdin_diff_with_changes_has_crlf_source_lines() {
    let space = Workspace::new();
    let mut command = space.cli();
    command.args(["--diff", "--line-ending=crlf-preserve"]);
    let (_, output) = command
        .pass_stdin(b"#let value  =  1\r\n")
        .spawn_with_info(None);

    assert_eq!(output.status.code(), Some(1));
    assert_eq!(
        output.stdout,
        b"@@ -1 +1 @@\n-#let value  =  1\r\n+#let value = 1\r\n\n"
    );
    assert!(output.stderr.is_empty());
}

#[test]
fn test_lf_and_crlf_preserve_leave_bare_cr_unchanged() {
    let space = Workspace::new();
    let input = b"#let a  =  0\r\n#let b  =  1\r#let c  =  2";
    let cases: [(&[&str], &[u8]); 2] = [
        (&[], b"#let a = 0\n#let b = 1\r#let c = 2\n"),
        (
            &["--line-ending=crlf-preserve"],
            b"#let a = 0\n#let b = 1\r#let c = 2\n",
        ),
    ];

    for (args, expected) in cases {
        let mut command = space.cli();
        command.args(args);
        let (_, output) = command.pass_stdin(input).spawn_with_info(None);

        assert!(output.status.success());
        assert_eq!(output.stdout, expected);
        assert!(output.stderr.is_empty());
    }
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
