mod common;

use common::TestBook;

#[test]
fn simple() {
    let test_book = TestBook::new("simple").expect("couldn't create book");

    assert!(test_book.chapter1_contains(r#"img src="d2/1.1.svg" alt="" />"#));
}

#[test]
fn simple_output_dir() {
    let test_book = TestBook::new("simple").expect("couldn't create book");

    let output = test_book.book.source_dir().join("d2/1.1.svg");
    dbg!(&output);
    assert!(output.exists());

    assert!(test_book.chapter1_contains(r#"img src="d2/1.1.svg" alt="" />"#));
}
