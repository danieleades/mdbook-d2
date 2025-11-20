use std::{io, process};

use clap::Parser;
use mdbook_d2::D2;
use mdbook_preprocessor::{errors::Error, Preprocessor};
use semver::{Version, VersionReq};

#[derive(clap::Parser)]
pub struct Args {
    #[clap(subcommand)]
    command: Option<Command>,
}

#[derive(clap::Parser)]
pub enum Command {
    Supports { renderer: String },
}

fn main() {
    let args = Args::parse();

    // Users will want to construct their own preprocessor here
    let preprocessor = D2;

    if let Some(Command::Supports { renderer }) = args.command {
        handle_supports(&preprocessor, &renderer);
    } else if let Err(e) = handle_preprocessing(&preprocessor) {
        eprintln!("{e}");
        process::exit(1);
    }
}

fn handle_preprocessing(pre: &dyn Preprocessor) -> Result<(), Error> {
    let (ctx, book) = mdbook_preprocessor::parse_input(io::stdin())?;

    let book_version = Version::parse(&ctx.mdbook_version)?;
    let version_req = VersionReq::parse(mdbook_preprocessor::MDBOOK_VERSION)?;

    if !version_req.matches(&book_version) {
        eprintln!(
            "Warning: The {} plugin was built against version {} of mdbook, but we're being \
             called from version {}",
            pre.name(),
            mdbook_preprocessor::MDBOOK_VERSION,
            ctx.mdbook_version
        );
    }

    let processed_book = pre.run(&ctx, book)?;
    serde_json::to_writer(io::stdout(), &processed_book)?;

    Ok(())
}

fn handle_supports(pre: &dyn Preprocessor, renderer: &str) -> ! {
    let supported = pre.supports_renderer(renderer);

    // Signal whether the renderer is supported by exiting with 1 or 0.
    match supported {
        Ok(true) => process::exit(0),
        Ok(false) => process::exit(1),
        Err(e) => {
            eprintln!("{e}");
            process::exit(1);
        }
    }
}
