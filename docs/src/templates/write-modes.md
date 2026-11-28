# Write modes

By default a `[[files]]` entry **overwrites** its destination. Set a `mode` on
the entry to merge the rendered content into a file that already exists instead,
so a template can ship incremental changes — extra rules, a config block, a diff
— that people apply on top of their own files without losing them.

Patch modes do **not** need `--force`; editing an existing file is the whole
point.

| `mode` | Behaviour | Extra fields |
|---|---|---|
| `overwrite` | Replace the file (default — same as omitting `mode`). | — |
| `append` | Add content to the **end** of the file (creates it if missing). | — |
| `prepend` | Add content to the **start** of the file. | — |
| `insert` | Insert on the line(s) before/after the **first** line containing `anchor`. Falls back to `append` when the anchor is absent. | `anchor`, `position` |
| `block` | Maintain an idempotent, marker-delimited block. Re-applying replaces the block in place. | `marker` |
| `patch` | Apply the content to the file as a **unified diff**. | — |

A newline always separates existing content from appended or prepended content.
`anchor` and `marker` are rendered like any other field. Patch modes apply only
to filesystem targets; for `stdout://`, `clipboard://`, and the like the
rendered fragment is written as-is.

## append

Add to the end of a file; nothing already there is touched.

```toml
[[files]]
path = "CLAUDE.md"
mode = "append"
content = """

## Added by the shared template
- Always run `cargo test` before committing.
"""
```

## insert

Place content relative to the first line matching `anchor`.

```toml
[[files]]
path = "CLAUDE.md"
mode = "insert"
anchor = "## Conventions"
position = "after"          # "before" | "after" (default: after)
content = "- Prefer `eprintln!` for progress messages."
```

## block

Maintain an idempotent block, delimited by
`<!-- spark:<marker> start -->` / `<!-- spark:<marker> end -->` comment markers.
Re-running updates the block instead of duplicating it.

```toml
[[files]]
path = "CLAUDE.md"
mode = "block"
marker = "agent-rules"      # optional; defaults to "spark"
content = """
- Never commit secrets.
- Keep functions small.
"""
```

## patch

`content` is a unified diff applied to the file.

```toml
[[files]]
path = "Cargo.toml"
mode = "patch"
content = """
--- a
+++ b
@@ -1,3 +1,3 @@
 [package]
-version = "1.0.0"
+version = "1.1.0"
 edition = "2021"
"""
```

> `--dry-run` prints the patch **fragment** (what gets merged) with its mode, not
> the final merged file — the merge reads the destination, which a dry run never
> touches.

## From the library

Set the mode with the `File` builder:

```rust
File::create(path, content).with_mode(FileMode::Append);
File::create(path, content)
    .with_mode(FileMode::Insert)
    .with_anchor("## Conventions")
    .with_position(Position::After);
File::create(path, content)
    .with_mode(FileMode::Block)
    .with_marker("agent-rules");
```

Ready-to-run templates for every mode live in [`examples/patch-modes/`][ex] —
each folder has a seed file and a template to run against it.

[ex]: https://github.com/pwnxpl0it/spark/tree/main/examples/patch-modes
