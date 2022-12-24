# D2 preprocessor for mdbook

A preprocessor for [mdbook](https://github.com/rust-lang/mdBook) to convert
`d2` codeblocks into SVG images using
[D2](https://github.com/terrastruct/d2).

## Installation

Install with cargo:

```sh
cargo install mdbook-d2
```

Or to install from git:

```sh
cargo install --git https://github.com/danieleades/mdbook-d2
```

## Requirements

This preprocessor assumes that `D2` is installed locally and on the path. D2 installation instructions can be found [here](https://github.com/terrastruct/d2#install).

Check that the local installation is working correctly using

```sh
d2 --version
```

## Usage

Add this to your `book.toml`:
```toml
[preprocessor.d2]
```

## Thanks

The code in this preprocessor is based on that from <https://github.com/matthiasbeyer/mdbook-svgbob2>
