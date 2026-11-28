# Spark

[![DeepWiki](https://img.shields.io/badge/DeepWiki-pwnxpl0it%2Fspark-blue.svg)](https://deepwiki.com/pwnxpl0it/spark)
![Latest Release](https://img.shields.io/github/v/release/pwnxpl0it/spark?label=Release)
![Build Status](https://img.shields.io/github/actions/workflow/status/pwnxpl0it/spark/rust.yml?branch=main)
![License](https://img.shields.io/github/license/pwnxpl0it/spark)

A TOML-based project initializer and template engine. Describe a project once as
a TOML template — the files, the layout, and the values that fill them — then
generate it with one command:

```sh
spark browser-extension
```

Templates are plain TOML, so they are easy to read, share, and version. Spark
runs as a CLI and ships as a Rust library, so the same templates work from the
terminal, an editor, or your own tools.

## Install

With Cargo:

```sh
cargo install --git https://github.com/pwnxpl0it/spark
```

Or grab a prebuilt binary from the [Releases][releases] page. Full instructions,
including the no-clipboard build, are in the [Installation guide][docs-install].

## Quick start

Templates live in `~/.config/spark/templates/`. Write one:

```toml
# ~/.config/spark/templates/service.toml
[options]
project_root = "{{$PROJECTNAME}}"

[[files]]
path = "{{$PROJECTNAME}}/main.rs"
content = "fn main() { println!(\"hello from {{$PROJECTNAME}}\"); }"
```

Generate from it:

```sh
spark service                               # prompts for PROJECTNAME
spark service --from "PROJECTNAME=billing"  # or supply it
spark service --dry-run                     # preview without writing
spark undo                                  # reverse the last run
```

The [Quick start][docs-quickstart] walks through this in full.

## What it does

- **TOML templates** — files, directories, and options in one file.
- **Placeholders and functions** — `{{$NAME}}`, `:read` prompts, `jq`-style JSON
  lookups.
- **Values from anywhere** — `--from`, JSON, environment variables, `.env`,
  config keywords.
- **Write modes** — overwrite, append, prepend, insert, block, or unified-diff
  patch for files that already exist.
- **Output targets** — files, `stdout`, `stderr`, or the clipboard.
- **Undo, hooks, Git init, Liquid, and a library API.**
- **Start from GitHub** — build a template from a remote repository.

## Documentation

The full guide is an mdBook under [`docs/`](docs/), published at
**[pwnxpl0it.github.io/spark][docs]**:

- [Template format][docs-format] · [Placeholders][docs-placeholders] ·
  [Supplying values][docs-values]
- [Write modes][docs-modes] · [Output targets][docs-targets] ·
  [Liquid][docs-liquid]
- [Command reference][docs-cli] · [Creating templates][docs-creating] ·
  [Hooks][docs-hooks]
- [Rust library API][docs-library]

## Using the library

```toml
[dependencies]
spark = { git = "https://github.com/pwnxpl0it/spark" }
```

```rust
use spark::{Template, Context};

let template = Template::from_str(r#"
[[files]]
path = "{{$SLUG}}/main.rs"
content = "// generated for {{$AUTHOR}}"
"#)?;

let ctx = Context::new()
    .with_var("SLUG", "my_crate")
    .with_var("AUTHOR", "Alice")
    .non_interactive();

let files = template.render(&ctx)?;        // render in memory, no I/O
```

See the [Rust API guide][docs-library] for scaffolding to disk, JSON data, and
error handling.

## Editor integration

[spark.nvim](https://github.com/pwnxpl0it/spark.nvim) drives Spark from Neovim.

## Contributing

Build with `cargo build`, test with `cargo test`, and run `cargo fmt` and
`cargo clippy` before opening a PR. See the [Development guide][docs-dev].

> This project was previously named `idkmng`. Some older issues may still use
> that name.

[releases]: https://github.com/pwnxpl0it/spark/releases
[docs]: https://pwnxpl0it.github.io/spark/
[docs-install]: docs/src/installation.md
[docs-quickstart]: docs/src/quickstart.md
[docs-format]: docs/src/templates/format.md
[docs-placeholders]: docs/src/templates/placeholders.md
[docs-values]: docs/src/templates/values.md
[docs-modes]: docs/src/templates/write-modes.md
[docs-targets]: docs/src/templates/output-targets.md
[docs-liquid]: docs/src/templates/liquid.md
[docs-cli]: docs/src/cli/reference.md
[docs-creating]: docs/src/cli/creating.md
[docs-hooks]: docs/src/cli/hooks.md
[docs-library]: docs/src/library/api.md
[docs-dev]: docs/src/reference/development.md
