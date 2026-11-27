# Output targets

By default a `[[files]]` `path` is a filesystem path. Prefix it with a scheme to
send the rendered output somewhere else.

| Scheme | Behaviour |
|---|---|
| *(none)* | Write to the filesystem. |
| `file://path` | Write to `path` on the filesystem (explicit). |
| `stdout://` | Write rendered content to **stdout**. |
| `stderr://` | Write rendered content to **stderr**. |
| `clipboard://` | Copy rendered content to the system clipboard. |

Unrecognized schemes (for example `ftp://`) are not treated as targets and fall
back to plain filesystem output.

All progress messages go to stderr, so a `stdout://` target can be piped
cleanly.

## Examples

Print to stdout instead of writing a file:

```toml
[[files]]
path = "stdout://"
content = "Hello {{$NAME}}!"
```

Send a message to stderr:

```toml
[[files]]
path = "stderr://"
content = "warning: {{$MESSAGE}}"
```

An explicit filesystem path:

```toml
[[files]]
path = "file://src/main.rs"
content = "fn main() {}"
```

Copy to the clipboard:

```toml
[[files]]
path = "clipboard://"
content = """
{ "api_key": "{{$API_KEY}}" }
"""
```

Mix targets freely in one template:

```toml
[[files]]
path = "src/main.rs"       # normal file
content = "fn main() {}"

[[files]]
path = "stdout://"         # also print a summary
content = "Generated {{$PROJECTNAME}}"
```

## Notes

- Windows drive-letter paths such as `C:\Users\foo` are never mistaken for
  protocol URIs — the scheme detector requires more than one character before the
  colon.
- All templating (keyword replacement, JSON paths, Liquid) runs **before** the
  output is dispatched. The scheme prefix must be a literal string after
  replacement; `stdout://{{$TARGET}}` is not a dynamic target selector.
- The clipboard target needs the `clipboard` Cargo feature (on by default). On a
  headless system, or in a build without the feature, the write fails with
  `Error::OutputWrite` rather than panicking.
