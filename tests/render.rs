mod common;

use common::TestBook;

#[test]
fn inline() {
    let test_book = TestBook::new("inline").expect("couldn't create book");

    assert!(test_book.chapter1_contains(r"<svg"));
    assert!(test_book.chapter1_contains(r"</svg>"));
    assert!(test_book.chapter1_contains(r"<rect"));
    assert!(test_book.chapter1_contains(r"</rect>"));
}

#[test]
fn simple() {
    let test_book = TestBook::new("simple").expect("couldn't create book");

    assert!(test_book.chapter1_contains(r#"img src="d2/1.1.svg" alt="">"#));
}

#[test]
fn simple_output_dir() {
    let test_book = TestBook::new("simple").expect("couldn't create book");

    let output = test_book.book.source_dir().join("d2/1.1.svg");

    assert!(output.exists());
    assert!(test_book.chapter1_contains(r#"img src="d2/1.1.svg" alt="">"#));
}

#[test]
fn custom_src() {
    let test_book = TestBook::new("custom-src").expect("couldn't create book");

    let output = test_book.book.source_dir().join("d2/1.1.svg");

    assert!(output.exists());
    assert!(test_book.chapter1_contains(r#"img src="d2/1.1.svg" alt="">"#));
}

#[test]
fn broken_diagram_fails_build_when_fail_on_error() {
    let Err(err) = TestBook::new("broken") else {
        panic!("build should fail when a d2 diagram is invalid and fail-on-error is set");
    };

    let msg = format!("{err:#}");
    assert!(
        msg.contains("failed to compile D2 diagram (Chapter 1, #1)"),
        "unexpected error: {msg}"
    );
}

#[test]
fn broken_diagram_succeeds_by_default() {
    let result = TestBook::new("broken-default");

    assert!(
        result.is_ok(),
        "build should succeed when a d2 diagram is invalid and fail-on-error is unset"
    );
}

#[test]
fn missing_binary_error_includes_context() {
    let Err(err) = TestBook::new("missing-binary") else {
        panic!("build should fail when the d2 binary is missing and fail-on-error is set");
    };

    let msg = format!("{err:#}");
    assert!(
        msg.contains("failed to run D2 binary '/nonexistent/bin/d2' (Chapter 1, #1)"),
        "unexpected error: {msg}"
    );
}
