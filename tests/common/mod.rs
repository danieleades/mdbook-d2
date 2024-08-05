use std::fs::{self, File};
use std::io::{self, Read};
use std::path::{Path, PathBuf};

use anyhow::Context;
use mdbook::MDBook;
use mdbook_d2::D2;
use tempfile::TempDir;

/// Returns the path to the test library directory
fn library() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/library")
}

/// Represents a test book for mdbook with D2 preprocessor.
/// 
/// Creates a new book root in a temporary directory by cloning a given source directory.
pub struct TestBook {
    /// Temporary directory where the book is copied and built
    _temp_dir: TempDir,
    /// The MDBook instance
    pub book: MDBook,
}

impl TestBook {
    /// Creates a new TestBook instance
    ///
    /// # Arguments
    ///
    /// * `book` - The name of the book in the test library
    ///
    /// # Returns
    ///
    /// A Result containing the TestBook instance or an error
    pub fn new(book: &str) -> anyhow::Result<Self> {
        let temp_dir = tempfile::tempdir().context("unable to create temporary directory")?;

        let source_book_root = library().join(book);

        recursive_copy(&source_book_root, temp_dir.path())
            .with_context(|| "Couldn't copy files into a temporary directory")?;

        let mut book = MDBook::load(temp_dir.path()).context("unable to load book from disk")?;

        book.with_preprocessor(D2)
            .build()
            .context("failed to build book")?;

        Ok(Self {
            _temp_dir: temp_dir,
            book,
        })
    }

    /// Checks if the first chapter of the book contains a specific snippet
    ///
    /// # Arguments
    ///
    /// * `snippet` - The text to search for in the chapter
    ///
    /// # Returns
    ///
    /// A boolean indicating whether the snippet was found
    pub fn chapter1_contains(&self, snippet: &str) -> bool {
        let chapter1 = self
            .book
            .root
            .join(&self.book.config.build.build_dir)
            .join("chapter1.html");
        dbg!(&chapter1);
        let mut content = String::new();
        File::open(chapter1)
            .expect("couldn't read chapter1.html")
            .read_to_string(&mut content)
            .unwrap();
        content.contains(snippet)
    }
}

/// Recursively copies an entire directory tree to another location
///
/// This function is similar to the `cp -r` command in Unix-like systems.
///
/// # Arguments
///
/// * `src` - The source directory path
/// * `dst` - The destination directory path
///
/// # Returns
///
/// An io::Result indicating success or failure of the copy operation
fn recursive_copy(src: impl AsRef<Path>, dst: impl AsRef<Path>) -> io::Result<()> {
    fs::create_dir_all(&dst)?;
    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let ty = entry.file_type()?;
        if ty.is_dir() {
            recursive_copy(entry.path(), dst.as_ref().join(entry.file_name()))?;
        } else {
            fs::copy(entry.path(), dst.as_ref().join(entry.file_name()))?;
        }
    }
    Ok(())
}
