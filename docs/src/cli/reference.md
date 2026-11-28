# Command reference

```text
spark [TEMPLATE] [OPTIONS]
spark <SUBCOMMAND> [OPTIONS]
```

Run a template by name (looked up in the templates directory) or by path. With
no template and no subcommand, Spark prints help.

## Subcommands

| Command | Description |
|---|---|
| `spark <template>` | Generate from a template. |
| `spark list` | List templates in the templates directory, with their descriptions. |
| `spark init` | Build a template from the current directory's files. See [Creating templates](creating.md). |
| `spark undo` | Reverse the most recent generation. See [Undo](undo.md). |

`spark new` is not a subcommand — `new` is the starter template Spark writes to
your templates directory on first run, so `spark new` generates from it.

## Options

These apply when generating from a template.

| Flag | Description |
|---|---|
| `-o, --output-dir <DIR>` | Generate into `DIR` (created if needed). |
| `-n, --dry-run` | Show what would be produced; write nothing. |
| `-f, --force` | Overwrite existing files without asking. |
| `--from <K=V,...>` | Supply placeholder values. Repeatable. |
| `--json <FILE>` | Load JSON data for `{{$.…}}` lookups. |
| `--git` | Initialize a Git repository, regardless of template options. |
| `--liquid` / `--no-liquid` | Force Liquid rendering on or off for this run. |
| `--strict` | Abort on an unresolved placeholder instead of warning. |
| `--allow-outside` | Permit destinations that escape the working directory. |
| `--run-hooks` | Execute the template's `pre`/`post` hooks. |
| `-c, --config <FILE>` | Use a specific config file. Works after subcommands too. |
| `-q, --quiet` | Reduce output. |

Every failure is printed as `error: …` and exits non-zero.

## Output directory

Files are created relative to the current directory by default. Point Spark
elsewhere with `-o`/`--output-dir`; the directory is created if needed and
relative `[[files]]` paths resolve from it. The template name and `--json` path
are still looked up relative to where you ran the command.

```sh
spark python -o ~/projects/new_app
```

## Dry run

See what a template would produce without touching the filesystem, the
clipboard, or git:

```sh
spark template --dry-run      # or -n
```

Each file is printed as a `==> path` header followed by its rendered content.
A dry run also lists any hook commands without executing them.

## Existing files

Spark checks every destination before writing anything. If a file already exists
it asks before overwriting, and refuses in non-interactive use. Pass
`--force`/`-f` to overwrite without asking:

```sh
spark template --force
```

This guard applies to overwrite-mode entries only. [Patch modes](../templates/write-modes.md)
are meant to edit existing files, so they skip the check and merge instead.
Library callers use `Context::force(true)`; otherwise `extract_with_context`
returns `Error::FileExists`.

## Writing outside the working directory

Destinations that can escape the working directory are refused before anything
is written, so a template you did not author cannot quietly write to `~/.bashrc`
or `/etc`. A path is rejected when, after `~`/`$VAR` expansion, it is **absolute**
or contains a `..` component — in every write mode, including the patch modes.

Pass `--allow-outside` (or `Context::allow_outside(true)`) to permit absolute and
`..` destinations. Do this only for templates you trust, since it lets a template
write anywhere you can.
