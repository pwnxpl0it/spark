# Placeholders and functions

A placeholder is `{{$NAME}}`. Spark resolves it from a value you supply (see
[Supplying values](values.md)), from one of the built-ins below, or by running a
function.

## Built-in placeholders

These are always available unless you override them:

| Placeholder | Value | Example |
|---|---|---|
| `PROJECTNAME` | Prompts for the project name (reused everywhere). | |
| `CURRENTDIR` | Current directory name. | `pwd=/foo/bar` → `bar` |
| `HOME` | Home directory. | `/home/user` |
| `YYYY` | Year, four digits. | `2026` |
| `YY` | Year, two digits. | `26` |
| `MM` | Month. | `2` |
| `DD` | Day. | `24` |
| `NOW` | Local date and time. | `2026-02-23 22:22:38.151 +00:00` |
| `NOW_UTC` | Date and time in UTC. | `2026-02-23 22:21:17.897 UTC` |

`PROJECTNAME` is special: it is resolved the first time any file, path, or
`project_root` references it, and the answer is reused for the whole run.

## Functions

A function runs when you write `{{$NAME:FUNCTION}}`.

| Function | Description | Example |
|---|---|---|
| `read` | Prompt for a value on stdin. | `{{$VAR:read}}` |
| `read("Label")` | Prompt with a custom label instead of the variable name. | `{{$VAR:read("Project name")}}` |

```toml
[[files]]
path = "example.txt"
content = """
User input: {{$USER_INPUT:read}}
"""
```

A `:read` value is resolved once and reused: if file 1 prompts for
`{{$TOKEN:read}}`, file 2's `{{$TOKEN}}` gets the same answer. A value already
supplied another way (environment variable, config keyword, `--from`, or JSON)
satisfies the `:read` without prompting.

> Because environment variables are placeholders, an exported variable silently
> satisfies a `:read` of the same name. See [Supplying values](values.md).
