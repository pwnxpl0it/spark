# Patch-mode example templates

Ready-to-run templates that exercise every non-overwrite write `mode`
(`append`, `prepend`, `insert`, `block`, `patch`) plus a `mixed` template that
combines several entries. Each folder is self-contained: a **seed** file (the
"existing" file a user already has) and a **template** that merges into it.

## Running one

From inside a mode's folder, run the template against its seed:

```sh
cd examples/patch-modes/append
spark ./append.toml            # edits CLAUDE.md in place — no --force needed
cat CLAUDE.md                  # see the merge
```

The seeds are tracked by git, so **reset them** after experimenting:

```sh
git checkout -- examples/patch-modes      # restore every seed
```

Prefer not to touch the repo copy? Work on a throwaway copy instead:

```sh
cp -r examples/patch-modes/append /path/to/scratch && cd /path/to/scratch
spark ./append.toml
```

Preview without writing anything:

```sh
spark ./append.toml --dry-run   # prints the fragment + its mode
```

## What each folder shows

| Folder    | Mode      | Demonstrates |
|-----------|-----------|--------------|
| `append`  | `append`  | Add rules to the end of an existing `CLAUDE.md`, no `--force`. |
| `prepend` | `prepend` | Add a new entry to the top of a `CHANGELOG.md`. |
| `insert`  | `insert`  | Place a key right after an anchor line; the anchor uses a `{{$SECTION}}` placeholder (`--from`). |
| `block`   | `block`   | Idempotent, marker-delimited block. **Run it twice** — the block is replaced in place, never duplicated. |
| `patch`   | `patch`   | Apply a unified diff to bump a value. |
| `mixed`   | several   | One template that both appends to `CLAUDE.md` and inserts into `settings.toml`, applied all-or-nothing. Pass `--from "TIMEOUT=30"`. |

## Notes

- A newline always separates existing content from merged content; a file keeps
  its own line endings (LF or CRLF).
- `insert` uses the **first** line matching `anchor`, and falls back to
  appending if the anchor is not found.
- `block` markers are HTML comments (`<!-- spark:<marker> start -->` …), which
  suit Markdown targets like `CLAUDE.md`.
- `patch` reports an error (and writes nothing) if the diff does not apply.
