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

    let output = test_book.book.source_dir().join("d2/1.1.svg");

    assert!(output.exists());
    assert!(test_book.chapter1_contains(r#"img src="d2/1.1.svg" alt="">"#));
}

#[test]
fn scenarios() {
    let test_book = TestBook::new("scenarios").expect("couldn't create book");
    let output = test_book.book.source_dir().join("d2/1.1");

    assert!(output.join("index.svg").exists());
    assert!(output.join("with_z.svg").exists());
    assert!(!test_book.chapter1_contains(r#"img src="d2/1.1/index.svg" alt="">"#));
    assert!(test_book.chapter1_contains("data-d2-version"));
}

#[test]
fn scenarios_embedded() {
    let test_book = TestBook::new("scenarios-embedded").expect("couldn't create book");
    let chapter1 = test_book.chapter1();

    let output = test_book.book.source_dir().join("d2/1.1");

    assert!(output.join("index.svg").exists());
    assert!(output.join("with_z.svg").exists());
    assert!(chapter1.contains("<svg"));
    assert!(!chapter1.contains(r#"img src="d2/1.1/index.svg" alt="">"#));
}

#[test]
fn custom_src() {
    let test_book = TestBook::new("custom-src").expect("couldn't create book");

    let output = test_book.book.source_dir().join("d2/1.1.svg");

    assert!(output.exists());
    assert!(test_book.chapter1_contains(r#"img src="d2/1.1.svg" alt="">"#));
}
