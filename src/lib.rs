//! [D2] diagram generator [`Preprocessor`] library for [`MdBook`](https://rust-lang.github.io/mdBook/).

#![deny(
    clippy::all,
    missing_debug_implementations,
    missing_copy_implementations,
    missing_docs,
    clippy::cargo
)]
#![warn(clippy::pedantic, clippy::nursery)]

use mdbook::{
    book::Book,
    errors::Error,
    preprocess::{Preprocessor, PreprocessorContext},
    BookItem,
};
use pulldown_cmark::{CodeBlockKind, CowStr, Event, Options, Parser, Tag};
use pulldown_cmark_to_cmark::cmark;

mod backend;
use backend::Backend;

mod config;

/// [D2] diagram generator [`Preprocessor`] for [`MdBook`](https://rust-lang.github.io/mdBook/).
#[derive(Default, Clone, Copy, Debug)]
pub struct D2;

impl Preprocessor for D2 {
    fn name(&self) -> &str {
        "d2"
    }

    fn run(&self, ctx: &PreprocessorContext, mut book: Book) -> Result<Book, Error> {
        let backend = Backend::from_context(ctx);

        for section in &mut book.sections {
            if let BookItem::Chapter(chapter) = section {
                let events = process_events(
                    &backend,
                    &chapter.name,
                    Parser::new_ext(&chapter.content, Options::all()),
                );

                // create a buffer in which we can place the markdown
                let mut buf = String::with_capacity(chapter.content.len() + 128);

                // convert it back to markdown and replace the original chapter's content
                cmark(events, &mut buf).unwrap();
                chapter.content = buf;
            }
        }

        Ok(book)
    }

    fn supports_renderer(&self, _renderer: &str) -> bool {
        true
    }
}

fn process_events<'a>(
    backend: &'a Backend,
    chapter: &'a str,
    events: impl Iterator<Item = Event<'a>> + 'a,
) -> impl Iterator<Item = Event<'a>> + 'a {
    let mut in_block = false;
    // if Windows crlf line endings are used, a code block will consist
    // of many different Text blocks, thus we need to buffer them in here
    // see https://github.com/raphlinus/pulldown-cmark/issues/507
    let mut diagram = String::new();
    let mut diagram_index = 0;

    events.flat_map(move |event| {
        match (&event, in_block) {
            // check if we are entering a d2 codeblock
            (
                Event::Start(Tag::CodeBlock(CodeBlockKind::Fenced(CowStr::Borrowed("d2")))),
                false,
            ) => {
                in_block = true;
                diagram.clear();
                diagram_index += 1;
                vec![]
            }
            // check if we are currently inside a d2 block
            (Event::Text(content), true) => {
                diagram.push_str(content);
                vec![]
            }
            // check if we are exiting a d2 block
            (Event::End(Tag::CodeBlock(CodeBlockKind::Fenced(CowStr::Borrowed("d2")))), true) => {
                in_block = false;
                backend.render(chapter, diagram_index, &diagram)
            }
            // if nothing matches, change nothing
            _ => vec![event],
        }
    })
}
