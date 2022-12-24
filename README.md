# D2 preprocessor for mdbook

[![codecov](https://codecov.io/gh/danieleades/mdbook-d2/branch/main/graph/badge.svg?token=BIHcAnynaN)](https://codecov.io/gh/danieleades/mdbook-d2)
[![Continuous integration](https://github.com/danieleades/mdbook-d2/actions/workflows/CI.yml/badge.svg)](https://github.com/danieleades/mdbook-d2/actions/workflows/CI.yml)

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
# path to d2 binary.
# optional. default is "d2" (ie. on the path).
path = "d2"
# layout engine for diagrams. See https://github.com/terrastruct/d2#plugins.
# optional. default is "dagre".
layout = "dagre"
# output directory relative to `src/` for generated diagrams.
# optional. default is "d2".
output-dir = "d2"
```

## Thanks

The code in this preprocessor is based on that from <https://github.com/matthiasbeyer/mdbook-svgbob2>
