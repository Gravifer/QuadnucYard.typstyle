mod common;

use common::{Workspace, typstyle_cmd_snapshot};

#[test]
fn test_one() {
    let mut space = Workspace::new();
    space.write_tracked("a.typ", "#let a  =  0");

    typstyle_cmd_snapshot!(space.cli().args(["a.typ"]), @r"
    success: true
    exit_code: 0
    ----- stdout -----
    #let a = 0

    ----- stderr -----
    ");

    assert!(space.all_unmodified());
}

#[test]
fn test_one_inplace() {
    let mut space = Workspace::new();
    space.write_tracked("a.typ", "#let a  =  0");

    typstyle_cmd_snapshot!(space.cli().args(["a.typ", "-i"]), @r"
    success: true
    exit_code: 0
    ----- stdout -----

    ----- stderr -----
    ");

    assert_eq!(space.read_string("a.typ"), "#let a = 0\n");
}

#[test]
fn test_one_quiet() {
    let mut space = Workspace::new();
    space.write_tracked("a.typ", "#let a  =  0");

    typstyle_cmd_snapshot!(space.cli().args(["a.typ", "-q"]), @r"
    success: true
    exit_code: 0
    ----- stdout -----
    #let a = 0

    ----- stderr -----
    ");

    assert!(space.all_unmodified());
}

#[test]
fn test_one_erroneous() {
    let mut space = Workspace::new();
    space.write_tracked("a.typ", "#let");

    typstyle_cmd_snapshot!(space.cli().args(["a.typ"]), @r"
    success: true
    exit_code: 0
    ----- stdout -----
    #let
    ----- stderr -----
    warn: Failed to parse a.typ. The source is erroneous.
    ");

    assert!(space.all_unmodified());
}

#[test]
fn test_one_inplace_erroneous() {
    let mut space = Workspace::new();
    space.write_tracked("a.typ", "#let");

    typstyle_cmd_snapshot!(space.cli().args(["a.typ", "-i"]), @r"
    success: true
    exit_code: 0
    ----- stdout -----

    ----- stderr -----
    warn: Failed to parse a.typ. The source is erroneous.
    ");

    assert!(space.all_unmodified());
}

#[test]
fn test_one_check_quiet() {
    let mut space = Workspace::new();
    space.write_tracked("a.typ", "#let a  =  0");

    typstyle_cmd_snapshot!(space.cli().args(["a.typ", "--check", "-q"]), @r"
    success: false
    exit_code: 1
    ----- stdout -----

    ----- stderr -----
    ");

    assert!(space.all_unmodified());
}

#[test]
fn test_default_lf_check_rejects_crlf() {
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
fn test_crlf_preserve_check_unchanged() {
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
fn test_crlf_preserve_check_formatted_content() {
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
fn test_crlf_preserve_diff_unchanged() {
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
fn test_crlf_preserve_inplace_preserves_line_endings() {
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

    assert_eq!(space.read_bytes("a.typ"), b"#let a = 0\r\n#let b = 1\r\n");

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
fn test_default_lf_file_output_remains_lf() {
    let mut space = Workspace::new();
    space.write_tracked("a.typ", b"#let a  =  0\r\n");

    let output = space.cli().arg("a.typ").output().unwrap();

    assert!(output.status.success());
    assert_eq!(output.stdout, b"#let a = 0\n");
    assert!(output.stderr.is_empty());
    assert!(space.all_unmodified());
}

#[test]
fn test_explicit_lf_inplace_normalizes_crlf() {
    let mut space = Workspace::new();
    space.write_tracked("a.typ", b"#let a  =  0\r\n#let b  =  1");

    typstyle_cmd_snapshot!(space.cli().args([
        "a.typ",
        "--inplace",
        "--line-ending=lf",
    ]), @r"
    success: true
    exit_code: 0
    ----- stdout -----

    ----- stderr -----
    ");

    assert_eq!(space.read_bytes("a.typ"), b"#let a = 0\n#let b = 1\n");
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
fn test_crlf_preserve_mixed_line_endings_fall_back_to_lf() {
    let mut space = Workspace::new();
    space.write_tracked("a.typ", b"#let a  =  0\r\n#let b  =  1\n");

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

    assert_eq!(space.read_bytes("a.typ"), b"#let a = 0\n#let b = 1\n");
}

#[test]
fn test_crlf_preserve_with_cr_at_eof_uses_formatter_output() {
    let mut space = Workspace::new();
    space.write_tracked("a.typ", b"#let a  =  0\r\n#let b  =  1\r");

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

    assert_eq!(space.read_bytes("a.typ"), b"#let a = 0\n#let b = 1\n");
}

#[test]
fn test_crlf_preserve_multiline_literals() {
    let mut space = Workspace::new();
    space.write_tracked(
        "a.typ",
        b"#let string  =  \"a\r\nb\"\r\n#let raw  =  ```a\r\nb```\r\n",
    );

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

    assert_eq!(
        space.read_bytes("a.typ"),
        b"#let string = \"a\r\nb\"\r\n#let raw = ```a\r\nb```\r\n"
    );
}

#[test]
fn test_crlf_preserve_does_not_rewrite_non_ascii_newline() {
    let mut space = Workspace::new();
    space.write_tracked("a.typ", "#let value  =  \"a\u{2028}b\"\r\n");

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

    assert_eq!(
        space.read_bytes("a.typ"),
        "#let value = \"a\u{2028}b\"\r\n".as_bytes()
    );
}

#[test]
fn test_two_0() {
    let mut space = Workspace::new();
    space.write_tracked("a.typ", "#let a = 0\n");
    space.write_tracked("b.typ", "#let b = 1\n");

    typstyle_cmd_snapshot!(space.cli().args(["a.typ", "b.typ"]), @r"
    success: true
    exit_code: 0
    ----- stdout -----
    #let a = 0
    #let b = 1

    ----- stderr -----
    ");

    assert!(space.all_unmodified());
}

#[test]
fn test_two_1() {
    let mut space = Workspace::new();
    space.write_tracked("a.typ", "#let a = 0\n");
    space.write_tracked("b.typ", "#let b  =  1\n");

    typstyle_cmd_snapshot!(space.cli().args(["a.typ", "b.typ"]), @r"
    success: true
    exit_code: 0
    ----- stdout -----
    #let a = 0
    #let b = 1

    ----- stderr -----
    ");

    assert!(space.all_unmodified());
}

#[test]
fn test_two_2() {
    let mut space = Workspace::new();
    space.write_tracked("a.typ", "#let a  =  0\n");
    space.write_tracked("b.typ", "#let b  =  1\n");

    typstyle_cmd_snapshot!(space.cli().args(["a.typ", "b.typ"]), @r"
    success: true
    exit_code: 0
    ----- stdout -----
    #let a = 0
    #let b = 1

    ----- stderr -----
    ");

    assert!(space.all_unmodified());
}

#[test]
fn test_two_0_inplace() {
    let mut space = Workspace::new();
    space.write_tracked("a.typ", "#let a = 0\n");
    space.write_tracked("b.typ", "#let b = 1\n");

    typstyle_cmd_snapshot!(space.cli().args(["a.typ", "b.typ", "-i"]), @r"
    success: true
    exit_code: 0
    ----- stdout -----

    ----- stderr -----
    ");

    assert!(space.is_unmodified("a.typ"));
    assert!(space.is_unmodified("b.typ"));
}

#[test]
fn test_two_1_inplace() {
    let mut space = Workspace::new();
    space.write_tracked("a.typ", "#let a = 0\n");
    space.write_tracked("b.typ", "#let b  =  1\n");

    typstyle_cmd_snapshot!(space.cli().args(["a.typ", "b.typ", "-i"]), @r"
    success: true
    exit_code: 0
    ----- stdout -----

    ----- stderr -----
    ");

    assert!(space.is_unmodified("a.typ"));
    assert_eq!(space.read_string("b.typ"), "#let b = 1\n");
}

#[test]
fn test_two_2_inplace() {
    let mut space = Workspace::new();
    space.write_tracked("a.typ", "#let a  =  0\n");
    space.write_tracked("b.typ", "#let b  =  1\n");

    typstyle_cmd_snapshot!(space.cli().args(["a.typ", "b.typ", "-i"]), @r"
    success: true
    exit_code: 0
    ----- stdout -----

    ----- stderr -----
    ");

    assert_eq!(space.read_string("a.typ"), "#let a = 0\n");
    assert_eq!(space.read_string("b.typ"), "#let b = 1\n");
}

#[test]
fn test_two_0_check() {
    let mut space = Workspace::new();
    space.write_tracked("a.typ", "#let a = 0\n");
    space.write_tracked("b.typ", "#let b = 1\n");

    typstyle_cmd_snapshot!(space.cli().args(["a.typ", "b.typ", "--check"]), @r"
    success: true
    exit_code: 0
    ----- stdout -----

    ----- stderr -----
    ");

    assert!(space.all_unmodified());
}

#[test]
fn test_two_1_check() {
    let mut space = Workspace::new();
    space.write_tracked("a.typ", "#let a = 0\n");
    space.write_tracked("b.typ", "#let b  =  1\n");

    typstyle_cmd_snapshot!(space.cli().args(["a.typ", "b.typ", "--check"]), @r"
    success: false
    exit_code: 1
    ----- stdout -----
    Would reformat: b.typ

    ----- stderr -----
    ");

    assert!(space.all_unmodified());
}

#[test]
fn test_two_2_check() {
    let mut space = Workspace::new();
    space.write_tracked("a.typ", "#let a  =  0\n");
    space.write_tracked("b.typ", "#let b  =  1\n");

    typstyle_cmd_snapshot!(space.cli().args(["a.typ", "b.typ", "--check"]), @r"
    success: false
    exit_code: 1
    ----- stdout -----
    Would reformat: a.typ
    Would reformat: b.typ

    ----- stderr -----
    ");

    assert!(space.all_unmodified());
}

#[test]
fn test_cwd() {
    let mut space = Workspace::new();
    space.write_tracked("a.typ", "#let a = 0\n");
    space.write_tracked("b.typ", "#let b  =  1\n");
    space.write_tracked("d/c.typ", "#let c  =  2\n");

    typstyle_cmd_snapshot!(space.cli().args(["./d/.."]), @r"
    success: true
    exit_code: 0
    ----- stdout -----
    #let a = 0
    #let b = 1
    #let c = 2

    ----- stderr -----
    ");

    assert!(space.all_unmodified());
}

#[test]
fn test_cwd_check() {
    let mut space = Workspace::new();
    space.write_tracked("a.typ", "#let a = 0\n");
    space.write_tracked("b.typ", "#let b  =  1\n");
    space.write_tracked("d/c.typ", "#let c  =  2\n");

    typstyle_cmd_snapshot!(space.cli().args(["./d/..", "--check"]), @r"
    success: false
    exit_code: 1
    ----- stdout -----
    Would reformat: b.typ
    Would reformat: d/c.typ

    ----- stderr -----
    ");

    assert!(space.all_unmodified());
}

#[test]
fn test_many() {
    let mut space = Workspace::new();
    space.write_tracked("a.typ", "#let a = 0\n");
    space.write_tracked("b.typ", "#let b  =  1\n");
    space.write_tracked("d/c.typ", "#let c  =  2\n");
    space.write_tracked("d/d/e.typ", "#let d  =  3\n");

    typstyle_cmd_snapshot!(space.cli().args(["a.typ", "b.typ", "d"]), @r"
    success: true
    exit_code: 0
    ----- stdout -----
    #let a = 0
    #let b = 1
    #let c = 2
    #let d = 3

    ----- stderr -----
    ");

    assert!(space.is_unmodified("a.typ"));
}

#[test]
fn test_many_inplace() {
    let mut space = Workspace::new();
    space.write_tracked("a.typ", "#let a = 0\n");
    space.write_tracked("b.typ", "#let b  =  1\n");
    space.write_tracked("d/c.typ", "#let c  =  2\n");
    space.write_tracked("d/d/e.typ", "#let d  =  3\n");

    typstyle_cmd_snapshot!(space.cli().args(["a.typ", "b.typ", "d", "-i"]), @r"
    success: true
    exit_code: 0
    ----- stdout -----

    ----- stderr -----
    ");

    assert!(space.is_unmodified("a.typ"));
    assert!(space.is_modified("b.typ"));
    assert!(space.is_modified("d/c.typ"));
    assert!(space.is_modified("d/d/e.typ"));
}

#[test]
fn test_many_check() {
    let mut space = Workspace::new();
    space.write_tracked("a.typ", "#let a = 0\n");
    space.write_tracked("b.typ", "#let b  =  1\n");
    space.write_tracked("d/c.typ", "#let c  =  2\n");
    space.write_tracked("d/d/e.typ", "#let d  =  3\n");

    typstyle_cmd_snapshot!(space.cli().args(["a.typ", "b.typ", "d", "--check"]), @r"
    success: false
    exit_code: 1
    ----- stdout -----
    Would reformat: b.typ
    Would reformat: d/c.typ
    Would reformat: d/d/e.typ

    ----- stderr -----
    ");

    assert!(space.all_unmodified());
}

#[test]
fn test_diff_single_line() {
    let mut space = Workspace::new();
    space.write_tracked("single.typ", "#let x=1+2");

    typstyle_cmd_snapshot!(space.cli().args(["single.typ", "--diff"]), @r"
    success: false
    exit_code: 1
    ----- stdout -----
    --- single.typ
    +++ single.typ
    @@ -1 +1 @@
    -#let x=1+2
    \ No newline at end of file
    +#let x = 1 + 2


    ----- stderr -----
    ");

    assert!(space.all_unmodified());
}

#[test]
fn test_diff_multiline() {
    let mut space = Workspace::new();
    space.write_tracked("multi.typ", "#let x=1\n#let y=2+3");

    typstyle_cmd_snapshot!(space.cli().args(["multi.typ", "--diff"]), @r"
    success: false
    exit_code: 1
    ----- stdout -----
    --- multi.typ
    +++ multi.typ
    @@ -1,2 +1,2 @@
    -#let x=1
    -#let y=2+3
    \ No newline at end of file
    +#let x = 1
    +#let y = 2 + 3


    ----- stderr -----
    ");

    assert!(space.all_unmodified());
}
