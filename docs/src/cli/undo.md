# Undo

Every run that writes to disk records how to reverse itself. `spark undo` rolls
back the **most recent** generation:

```sh
spark my-template      # writes files (and records an undo manifest)
spark undo             # deletes files it created, restores files it overwrote
```

- Files Spark **created** are deleted.
- Files Spark **overwrote** or merged into with any non-overwrite mode
  (`append`, `prepend`, `insert`, `block`, `patch`) are restored to their exact
  previous content.
- Directories Spark created are removed when undo leaves them empty (deepest
  first). A directory you already had, or one that still holds files, is kept.
- `stdout://`, `stderr://`, and `clipboard://` targets are not tracked — there is
  nothing on disk to reverse.

The manifest is stored next to your config file, so your generated project stays
clean. It holds only the last run: `undo` reverses the latest generation and then
clears itself.

> Library callers get the same information from `Template::extract_with_undo`,
> which returns an `UndoLog` alongside the rendered files; call `UndoLog::apply()`
> to reverse the writes.
