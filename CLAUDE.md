# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## What this is

Spark is a Rust project initializer: it reads a TOML template (`[info]`, `[options]`, `[[files]]`), resolves `{{$PLACEHOLDER}}` tokens, and writes the resulting files. It ships as both a library crate (`src/core/lib.rs`) and a CLI binary (`src/cli/main.rs`), both named `spark`. The project was previously called `idkmng`; old issues may use that name.

## Commands

```sh
cargo build
cargo test                                   # all unit + doc tests
cargo test keywords                          # tests whose name contains "keywords"
cargo test templates::tests::extract_stderr_target_does_not_write_file   # one test
cargo fmt --all -- --check                   # CI gate
cargo clippy --all-targets -- -D warnings    # CI gate, also run with --no-default-features
cargo run -- <template> [-o DIR] [-n|--dry-run] [-f|--force] [--git] [--liquid|--no-liquid] [--strict] [--allow-outside] [-q] [--from "k=v,k2=v2"]... [--json FILE] [-c config.toml]
cargo run -- init                            # generate <cwd-name>.toml from files in cwd
cargo run -- list                            # list templates next to the config file
```

CI (`.github/workflows/rust.yml`) runs fmt, clippy (default and `--no-default-features`), and `cargo test` on a single Ubuntu runner with `RUSTFLAGS=-Dwarnings`; binaries for all platforms are only built by `release.yml` when a release is published. `build.rs` embeds Windows VERSIONINFO via `winresource` (Windows-only build dependency) to reduce Defender/SmartScreen false positives. Linux needs `libwayland-dev` for the `arboard` clipboard dependency, which sits behind the default `clipboard` cargo feature.

Tests live in `#[cfg(test)]` modules inside each source file (no `tests/` dir; it is gitignored). Filesystem fixtures use `tempfile::tempdir()`; never write into the repo cwd or change the process cwd from a test, because tests run in parallel.

## Architecture

### Rendering pipeline (`src/core/templates/mod.rs`)

`Template::render_inner` is the single pipeline every entry point goes through. For each `[[files]]` entry, in order:

1. `Fns::find_and_resolve` (`src/core/funcs.rs`) scans `content + "\n" + path` with `Keywords::regex()` (`\{\{\$.*?\}\}`) for placeholders not already in the keyword map. For each: copy from the base keyword if `{{$X:read}}` and `{{$X}}` is known; evaluate as a jq filter via `jaq` if the name contains `.` and JSON data is present; prompt on stdin for `:read` / `:read("Label")`; otherwise apply the context's `MissingPolicy` (`Empty` warns and inserts `""`, `Keep` leaves the token, `Error` fails). An unknown function name is `Error::InvalidFunction`.
2. `resolve_project_name` prompts for (or reads) `{{$PROJECTNAME}}` the first time any file, path, or `project_root` references it, and sets `options.project_root` from it.
3. `prepare_file_content` substitutes every keyword in one `regex::replace_all` pass, runs Liquid over the content only if `use_liquid` is `true` (default **false**), then unescapes `{{$$` to `{{$`.

Resolution is cumulative: a `:read` answered in file 1 is reused by file 2. Keyword map keys are always the full token form `{{$NAME}}`; `Keywords::from(name, func)` builds them and `Keywords::strip` reverses it. `{{$$NAME}}` is an escaped placeholder that renders as the literal `{{$NAME}}`; `spark init` escapes captured content this way.

Public entry points: `render` (in-memory, no side effects), `extract_with_context` (render + guards + write + git), and `extract(&mut HashMap)` (legacy wrapper that copies resolved values back into the caller's map). `extract_inner` runs two guards before writing anything: paths with `..` components fail with `Error::UnsafePath` unless `Context::allow_outside`, and existing destination files fail with `Error::FileExists` (or prompt when interactive) unless `Context::force`.

`Context` (`src/core/context.rs`) carries keywords, optional JSON, `interactive`, `on_missing`, `force`, and `allow_outside`. Non-interactive mode returns `Error::MissingVariable` instead of blocking on stdin.

### Output dispatch (`src/core/output_target.rs`)

Rendered paths are parsed by `OutputTarget::from_path`: `stdout://`, `stderr://`, `clipboard://` (exact match only), `file://path`, else a filesystem path. The file writer expands `~` and `$VAR` with `shellexpand` and creates parent dirs. All progress messages go to stderr so `stdout://` output can be piped; only rendered content and `spark list` results go to stdout.

### Keyword sources, in override order (lowest to highest)

`Keywords::init` (`src/core/keywords.rs`): `HOME`, `CURRENTDIR`, `NOW`, `NOW_UTC`, `YYYY`, `YY`, `MM`, `DD`, empty `PROJECTNAME`, then **every environment variable** (`.env` is loaded via `dotenvy` first). Then the `[Keywords]` table from the config file, then `--from` pairs. `--json` / `[options.json_data]` feeds the `{{$.path}}` lookups rather than the keyword map. Because env vars are keywords, a `{{$NAME:read}}` is silently satisfied by an exported `NAME`.

### CLI (`src/cli/`)

`args.rs` is a clap 4 derive struct; `--config` is global so it works after subcommands. `main` returns `ExitCode` and every failure is printed as `error: ...` with exit 1. `main.rs` resolves `<template>` by trying it as a local path first, then `<config_dir>/templates/<name>.toml`; template and `--json` paths are resolved before `--output-dir` changes the cwd. `Config::get_keywords` is read-only; `Config::ensure_default_exists` scaffolds the config dir only when the path is the default one. `--git` only sets `project_root` to `{{$PROJECTNAME}}` when the template left it empty. Git init (`templates/options/git/mod.rs`) uses `Command::current_dir`, never `set_current_dir`.

## Conventions

- `rustfmt.toml` sets `tab_spaces = 4`. Commit messages use a leading emoji plus a conventional-commit type, e.g. `🐛 fix: ...`, `✨ feat: ...`, `🧪 test: ...`, `🧹 chore: ...`.
- The README's placeholder table, functions table, options table, output-target table, and CLI flag sections are the user-facing contract; update them when changing any of those.
- Liquid uses the same `{{ }}` delimiters as Spark, so docs tell users to wrap Spark tokens as `{{ "{{$X}}" | filter }}` inside Liquid expressions. Escaped tokens (`{{$$X}}`) also need Liquid off or the same wrapping.
