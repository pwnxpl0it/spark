# Development

Build, test, and lint:

```sh
cargo build
cargo test                                   # unit + doc tests
cargo fmt --all -- --check
cargo clippy --all-targets -- -D warnings
cargo run -- [TEMPLATE] [OPTIONS]
```

CI runs fmt, clippy (with default features and `--no-default-features`), and the
tests on a single Linux runner for every push and pull request. Cross-platform
binaries are built only by the release workflow when a GitHub release is
published.

Tests live in `#[cfg(test)]` modules inside each source file. They cover
keywords, functions, utilities, templates (JSON paths, Liquid ordering,
escaping, and the overwrite and path guards), options, config, and the CLI.

## Building these docs

The documentation is an [mdBook][mdbook]. Install it and serve locally:

```sh
cargo install mdbook
mdbook serve docs --open      # live-reload at http://localhost:3000
mdbook build docs             # render to docs/book/
```

[mdbook]: https://rust-lang.github.io/mdBook/
