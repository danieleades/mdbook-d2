use std::path::{Path, PathBuf};
use std::{env, fs, io};

use mdbook::MDBook;
use mdbook_d2::D2;

struct ChangeDirGuard {
    original: PathBuf,
}

impl ChangeDirGuard {
    fn change_dir(target: &Path) -> io::Result<Self> {
        let original = env::current_dir()?;
        env::set_current_dir(target)?;
        Ok(Self { original })
    }
}

impl Drop for ChangeDirGuard {
    fn drop(&mut self) {
        let _ = env::set_current_dir(&self.original);
    }
}

#[test]
fn test_d2_preprocessor_integration() {
    // Set up the test book
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("test_book");

    let _dir_guard = ChangeDirGuard::change_dir(&root).expect("failed to change directory");

    // Load the book
    let mut md = MDBook::load(&root).expect("failed to load book");

    // Register the D2 preprocessor
    md.with_preprocessor(D2);

    // Build the book
    md.build().expect("failed to build book");

    // Check the output
    let html_file = root.join("book").join("index.html");
    let html_content = fs::read_to_string(html_file).expect("failed to read html content");

    // Make assertions about the processed content
    assert!(html_content.contains(r#"img src="d2/1.1.svg" alt="" />"#));
}
