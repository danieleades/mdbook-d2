//! [D2] diagram generator [`Preprocessor`] library for [`MdBook`](https://rust-lang.github.io/mdBook/).

#![deny(
    clippy::all,
    missing_debug_implementations,
    missing_copy_implementations,
    missing_docs
)]
#![warn(clippy::pedantic, clippy::nursery)]

use mdbook_preprocessor::{
    book::{Book, BookItem, Chapter},
    errors::Error,
    Preprocessor, PreprocessorContext,
};
use pulldown_cmark::{CodeBlockKind, CowStr, Event, Options, Parser, Tag, TagEnd};
use pulldown_cmark_to_cmark::{calculate_code_block_token_count, cmark_with_options};

mod backend;
use backend::{Backend, RenderContext};

mod config;

/// [D2] diagram generator [`Preprocessor`] for [`MdBook`](https://rust-lang.github.io/mdBook/).
#[derive(Default, Clone, Copy, Debug)]
pub struct D2;

impl Preprocessor for D2 {
    fn name(&self) -> &'static str {
        "d2"
    }

    fn run(&self, ctx: &PreprocessorContext, mut book: Book) -> Result<Book, Error> {
        let backend = Backend::from_context(ctx);

        book.for_each_mut(|section| {
            if let BookItem::Chapter(chapter) = section {
                let events: Vec<_> = process_events(
                    &backend,
                    chapter,
                    Parser::new_ext(&chapter.content, Options::all()),
                )
                .collect();

                // Determine the minimum number of backticks needed for code blocks.
                // Use 3 (the CommonMark default) unless nested code blocks require more.
                // This preserves the original markdown structure while correctly handling
                // code blocks that contain other code block examples.
                // See: https://github.com/danieleades/mdbook-d2/issues/170
                let code_block_token_count = calculate_code_block_token_count(events.iter())
                    .unwrap_or(3);

                let options = pulldown_cmark_to_cmark::Options {
                    code_block_token_count,
                    ..Default::default()
                };

                // create a buffer in which we can place the markdown
                let mut buf = String::with_capacity(chapter.content.len() + 128);

                // convert it back to markdown and replace the original chapter's content
                cmark_with_options(events.into_iter(), &mut buf, options).unwrap();
                chapter.content = buf;
            }
        });

        Ok(book)
    }
}

fn process_events<'a>(
    backend: &'a Backend,
    chapter: &'a Chapter,
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
            (Event::End(TagEnd::CodeBlock), true) => {
                in_block = false;
                let render_context = RenderContext::new(
                    chapter.source_path.as_ref().unwrap(),
                    &chapter.name,
                    chapter.number.as_ref(),
                    diagram_index,
                );
                backend
                    .render(&render_context, &diagram)
                    .unwrap_or_else(|e| {
                        // if we cannot render the diagram, print the error and return an empty
                        // block
                        eprintln!("{e}");
                        vec![]
                    })
            }
            // if nothing matches, change nothing
            _ => vec![event],
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Helper to round-trip markdown like the preprocessor does.
    fn round_trip_markdown(input: &str) -> String {
        let events: Vec<_> = Parser::new_ext(input, Options::all()).collect();
        let code_block_token_count =
            calculate_code_block_token_count(events.iter()).unwrap_or(3);
        let options = pulldown_cmark_to_cmark::Options {
            code_block_token_count,
            ..Default::default()
        };
        let mut output = String::new();
        cmark_with_options(events.into_iter(), &mut output, options).unwrap();
        output
    }

    /// Tests that code blocks preserve 3 backticks after round-trip conversion.
    ///
    /// This is a regression test for <https://github.com/danieleades/mdbook-d2/issues/170>.
    /// When using the default pulldown-cmark-to-cmark options, code blocks would be
    /// converted to use 4 backticks instead of 3, causing issues with other preprocessors.
    #[test]
    fn code_blocks_preserve_backticks() {
        let input = "```rust\nfn main() {}\n```\n";
        let output = round_trip_markdown(input);

        assert!(
            output.contains("```rust"),
            "expected 3 backticks, got: {output}"
        );
        assert!(
            !output.contains("````"),
            "should not have 4 backticks: {output}"
        );
    }

    #[test]
    fn multiple_code_blocks_preserve_backticks() {
        let input = r#"# Title

```rust
fn main() {}
```

Some text.

```python
print("hello")
```
"#;

        let output = round_trip_markdown(input);

        // Count occurrences of ``` (but not ````)
        let three_backticks = output.matches("```").count();
        let four_backticks = output.matches("````").count();

        assert_eq!(
            three_backticks, 4,
            "expected 4 occurrences of 3 backticks (2 code blocks × 2), got: {output}"
        );
        assert_eq!(
            four_backticks, 0,
            "should not have any 4 backticks: {output}"
        );
    }

    /// Test that code blocks containing backticks are properly escaped with more backticks.
    #[test]
    fn nested_code_blocks_escaped_correctly() {
        // A code block containing a literal 3-backtick code block example
        let input = r"Here's how to write a code block:

````markdown
```rust
fn main() {}
```
````

That's it!
";

        let output = round_trip_markdown(input);

        // The outer code block should still use 4 backticks to escape the inner 3
        assert!(
            output.contains("````"),
            "should have 4 backticks to escape nested block: {output}"
        );
        // The inner code block should be preserved as content
        assert!(
            output.contains("```rust"),
            "inner code block should be preserved: {output}"
        );
    }
}
