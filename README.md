# D2 preprocessor for mdbook

[![codecov](https://codecov.io/gh/danieleades/mdbook-d2/branch/main/graph/badge.svg?token=BIHcAnynaN)](https://codecov.io/gh/danieleades/mdbook-d2)
[![Continuous integration](https://github.com/danieleades/mdbook-d2/actions/workflows/CI.yml/badge.svg)](https://github.com/danieleades/mdbook-d2/actions/workflows/CI.yml)

A preprocessor for [mdbook](https://github.com/rust-lang/mdBook) to convert
`d2` codeblocks into SVG images using
[D2](https://github.com/terrastruct/d2).

## Installation

Install with cargo:

```sh
cargo install mdbook-d2 --locked
```

Or to install from git:

```sh
cargo install --git https://github.com/danieleades/mdbook-d2 --locked
```

## Requirements

This preprocessor assumes that `D2` is installed locally and on the path. D2 installation instructions can be found [here](https://github.com/terrastruct/d2#install).

Check that the local installation is working correctly using

```sh
d2 --version
```

## Usage

### Configuration

Add this to your `book.toml`:

```toml
[preprocessor.d2]

# path to d2 binary.
# optional. default is "d2" (ie. on the path).
path = "d2"

# layout engine for diagrams. See https://github.com/terrastruct/d2#plugins.
# optional. default is "dagre".
layout = "dagre"

# whether to use inline svg when rendering.
# if 'false', separate files will be generated in src/<output-dir> and referenced.
# optional. default is 'true'
inline = true

# output directory relative to `src/` for generated diagrams.
# This is ignored if 'inline' is 'true'.
# optional. default is "d2".
output-dir = "d2"
```

### Code Blocks

Use in your 'book' by annotating D2 code blocks-

````md

## My Diagram

```d2
# Actors
hans: Hans Niemann

defendants: {
  mc: Magnus Carlsen
  playmagnus: Play Magnus Group
  chesscom: Chess.com
  naka: Hikaru Nakamura

  mc -> playmagnus: Owns majority
  playmagnus <-> chesscom: Merger talks
  chesscom -> naka: Sponsoring
}

# Accusations
hans -> defendants: 'sueing for $100M'

# Offense
defendants.naka -> hans: Accused of cheating on his stream
defendants.mc -> hans: Lost then withdrew with accusations
defendants.chesscom -> hans: 72 page report of cheating
```
````

The code block will be replaced with the D2 diagram in the rendered document.

## Thanks

The code in this preprocessor is based on that from <https://github.com/matthiasbeyer/mdbook-svgbob2>

---

*Was this useful? [Buy me a coffee](https://github.com/sponsors/danieleades/sponsorships?sponsor=danieleades&preview=true&frequency=recurring&amount=5)*
