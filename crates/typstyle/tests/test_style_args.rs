mod common;

use common::{Workspace, typstyle_cmd_snapshot};

#[test]
fn test_tab_width() {
    let space = Workspace::new();

    let stdin = "#let f(x) = {
for i in range(0, 5) {
     x = x + i
 }
}";

    typstyle_cmd_snapshot!(space.cli().args(["-t=4"]).pass_stdin(stdin), @r"
    success: true
    exit_code: 0
    ----- stdout -----
    #let f(x) = {
        for i in range(0, 5) {
            x = x + i
        }
    }

    ----- stderr -----
    ");
}

#[test]
fn test_reorder_import_items() {
    let space = Workspace::new();

    let stdin = r#"#import "module.typ": xyz, func as renamed, h.i.j, a.b.c"#;

    typstyle_cmd_snapshot!(space.cli().pass_stdin(stdin), @r#"
    success: true
    exit_code: 0
    ----- stdout -----
    #import "module.typ": a.b.c, func as renamed, h.i.j, xyz

    ----- stderr -----
    "#);
    typstyle_cmd_snapshot!(space.cli().args(["--no-reorder-import-items"]).pass_stdin(stdin), @r#"
    success: true
    exit_code: 0
    ----- stdout -----
    #import "module.typ": xyz, func as renamed, h.i.j, a.b.c

    ----- stderr -----
    "#);
}

#[test]
fn test_wrap_text() {
    let space = Workspace::new();

    let stdin = "lorem  ipsum   dolor sit amet, consectetur   adipiscing elit.";

    typstyle_cmd_snapshot!(space.cli().args(["-c=20", "--wrap-text"]).pass_stdin(stdin), @r"
    success: true
    exit_code: 0
    ----- stdout -----
    lorem ipsum dolor
    sit amet,
    consectetur
    adipiscing elit.

    ----- stderr -----
    ");
}

#[test]
fn test_wrap_text_modes() {
    let space = Workspace::new();

    let stdin =
        "First sentence has   extra spaces and enough words to wrap. Second sentence follows.";

    typstyle_cmd_snapshot!(space.cli().args(["-c=34"]).pass_stdin(stdin), @r"
    success: true
    exit_code: 0
    ----- stdout -----
    First sentence has   extra spaces and enough words to wrap. Second sentence follows.

    ----- stderr -----
    ");
    typstyle_cmd_snapshot!(space.cli().args(["-c=34", "--wrap-text=none"]).pass_stdin(stdin), @r"
    success: true
    exit_code: 0
    ----- stdout -----
    First sentence has   extra spaces and enough words to wrap. Second sentence follows.

    ----- stderr -----
    ");
    typstyle_cmd_snapshot!(space.cli().args(["-c=34", "--wrap-text=fill"]).pass_stdin(stdin), @r"
    success: true
    exit_code: 0
    ----- stdout -----
    First sentence has extra spaces
    and enough words to wrap. Second
    sentence follows.

    ----- stderr -----
    ");
    typstyle_cmd_snapshot!(space.cli().args(["-c=34", "--wrap-text=sentence"]).pass_stdin(stdin), @r"
    success: true
    exit_code: 0
    ----- stdout -----
    First sentence has extra spaces and enough words to wrap.
    Second sentence follows.

    ----- stderr -----
    ");
    typstyle_cmd_snapshot!(space.cli().args(["-c=34", "--wrap-text=fill-sentence"]).pass_stdin(stdin), @r"
    success: true
    exit_code: 0
    ----- stdout -----
    First sentence has extra spaces
    and enough words to wrap.
    Second sentence follows.

    ----- stderr -----
    ");
}

#[test]
fn test_wrap_text_does_not_consume_input_path() {
    let space = Workspace::new();
    space.write("input.typ", "First sentence. Second sentence.");

    typstyle_cmd_snapshot!(space.cli().args(["--wrap-text", "input.typ"]), @r"
    success: true
    exit_code: 0
    ----- stdout -----
    First sentence. Second sentence.

    ----- stderr -----
    ");
}

#[test]
fn test_wrap_text_rejects_invalid_value() {
    let space = Workspace::new();

    typstyle_cmd_snapshot!(space.cli().args(["--wrap-text=bogus"]).pass_stdin(""), @r"
    success: false
    exit_code: 2
    ----- stdout -----

    ----- stderr -----
    error: invalid value 'bogus' for '--wrap-text[=<WRAP_TEXT>]'
      [possible values: none, fill, sentence, fill-sentence]

    For more information, try '--help'.
    ");
}

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
fn test_invalid_line_ending_mode() {
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
