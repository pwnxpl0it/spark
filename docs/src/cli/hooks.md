# Hooks

A template can declare shell commands to run before and after generation in a
top-level `[hooks]` table. `pre` commands run before any file is written; `post`
commands run after every file is written (and after git init, if enabled).

```toml
[hooks]
pre  = ["command -v cargo"]            # bail out early if a tool is missing
post = ["cargo fmt", "cargo build"]    # finalise the generated project

[[files]]
path = "src/main.rs"
content = "fn main() {}"
```

## Hooks are opt-in

Hooks run arbitrary shell commands, so Spark **never runs them unless you ask**.
Generating a template that defines hooks does nothing by default except print a
note:

```sh
spark my-template
# note: template defines 2 hook(s); re-run with --run-hooks to execute them
```

Pass `--run-hooks` to execute them:

```sh
spark my-template --run-hooks
```

This keeps you in control when generating from a template you did not write:
review it first, then opt in. Commands run through the platform shell (`sh -c`
on Unix, `cmd /C` on Windows).

## Working directory

`pre` commands run in the directory Spark runs in (the `-o DIR` target when
given, otherwise the current directory) — the project does not exist yet.

`post` commands run **inside the template's `project_root`** (the generated
project), so `cargo build`, `npm install`, and similar finalizers act on the new
project. When the template sets no `project_root`, `post` commands run in the
directory Spark runs in.

## Reviewing hooks first

`--dry-run` lists a template's hook commands and never executes them:

```sh
spark my-template --dry-run
# ...
# hooks (skipped without --run-hooks)
#   pre  command -v cargo
#   post cargo fmt
#   post cargo build
```

## Failure behavior

Commands in a phase run in order and stop at the first one that exits non-zero.
A failing `pre` command aborts before any file is written; a failing `post`
command surfaces as `error:` with a non-zero exit, after the files are on disk.
Hook side effects are external and are not tracked or reversed by Spark.
