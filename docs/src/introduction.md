# Introduction

Spark is a project initializer and template engine. You describe a project once
as a TOML template — the files, the directory layout, and the values that fill
them — and generate a ready-to-build project with a single command:

```sh
spark browser-extension
```

Templates are plain TOML, so they are easy to read, share, and keep in version
control. Spark runs as a standalone CLI and ships as a Rust library, so the same
templates work from the terminal, an editor, or your own tools.

## What it does

- **TOML templates.** Files, directories, and options live in one declarative
  file.
- **Placeholders and functions.** `{{$NAME}}` tokens, interactive `:read`
  prompts, and `jq`-style JSON lookups.
- **Values from anywhere.** `--from key=value`, a JSON file, environment
  variables, `.env` files, and config keywords — no prompts required.
- **Write modes.** Overwrite a file, or append, prepend, insert, maintain an
  idempotent block, or apply a unified-diff patch to a file that already exists.
- **Output targets.** Write to files, `stdout`, `stderr`, or the clipboard.
- **Undo.** `spark undo` reverses the last generation.
- **Hooks.** Opt-in shell commands that run before and after generation.
- **Start from GitHub.** Build a template from a remote repository.
- **Git integration.** Initialize a repository as part of project creation.
- **Liquid.** Conditionals, loops, and filters for advanced templates.
- **Library + CLI.** Use Spark from the shell or embed the crate.
- **Editor integration.** A Neovim plugin, [spark.nvim][nvim].

Start with [Installation](installation.md), then the
[Quick start](quickstart.md).

> This project was previously named `idkmng`. Some older issues and references
> may still use that name.

[nvim]: https://github.com/pwnxpl0it/spark.nvim
