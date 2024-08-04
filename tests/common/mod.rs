use std::fs::{self, File};
use std::io::{self, Read};
use std::path::{Path, PathBuf};

use anyhow::Context;
use mdbook::MDBook;
use mdbook_d2::D2;
use tempfile::TempDir;

fn library() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/library")
}

pub struct TestBook {
    _temp_dir: TempDir,
    book: MDBook,
}

impl TestBook {
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

/// Recursively copy an entire directory tree to somewhere else (a la `cp -r`).
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
