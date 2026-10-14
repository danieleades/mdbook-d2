mod common;

use common::TestBook;
use test_case::test_case;

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

#[test]
fn target_selects_a_single_board() {
    let test_book = TestBook::new("scenarios").expect("couldn't create book");

    let chapter1 = test_book.chapter1();
    assert_eq!(chapter1.matches("data-d2-version").count(), 2);

    // only the scenario adds 'zed'; the root board doesn't have it
    assert_eq!(chapter1.matches(">zed</text>").count(), 1);
}

#[test]
fn multiple_boards_without_target_is_an_error() {
    let Err(err) = TestBook::new("multiboard") else {
        panic!("build should fail when a diagram with multiple boards has no target");
    };

    let msg = format!("{err:#}");
    assert!(
        msg.contains("D2 diagram (Chapter 1, #1) has multiple boards"),
        "unexpected error: {msg}"
    );
}

#[test_case("empty"; "inline")]
#[test_case("empty-embedded"; "embedded")]
fn empty_diagram_is_an_error(book: &str) {
    let Err(err) = TestBook::new(book) else {
        panic!("build should fail when a diagram is empty and fail-on-error is set");
    };

    let msg = format!("{err:#}");
    assert!(
        msg.contains("D2 diagram (Chapter 1, #1) is empty"),
        "unexpected error: {msg}"
    );
}
