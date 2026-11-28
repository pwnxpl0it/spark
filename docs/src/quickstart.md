# Quick start

This walks through writing a template, generating from it, and undoing the
result.

## 1. Where templates live

Spark reads templates from the directory next to its config file, by default
`~/.config/spark/templates/`. On first run Spark scaffolds this directory and
writes a starter template named `new`, so `spark new` generates from it.

List what you have:

```sh
spark list
```

## 2. Write a template

Create `~/.config/spark/templates/service.toml`:

```toml
[info]
name = "service"
description = "A tiny service skeleton"

[options]
project_root = "{{$PROJECTNAME}}"

[[files]]
path = "{{$PROJECTNAME}}/main.rs"
content = """
// {{$PROJECTNAME}} — generated {{$YYYY}}-{{$MM}}-{{$DD}}
fn main() {
    println!("hello from {{$PROJECTNAME}}");
}
"""

[[files]]
path = "{{$PROJECTNAME}}/README.md"
content = "# {{$PROJECTNAME}}\n\n{{$DESCRIPTION:read}}\n"
```

- `{{$PROJECTNAME}}` is prompted for once and reused everywhere, including the
  directory name.
- `{{$YYYY}}`, `{{$MM}}`, `{{$DD}}` are built-in date values.
- `{{$DESCRIPTION:read}}` prompts for a value the first time it is needed.

## 3. Generate

```sh
spark service
```

Spark asks for the project name and description, then writes the files under a
new directory named after the project. To skip the prompts, supply the values:

```sh
spark service --from "PROJECTNAME=billing, DESCRIPTION=Billing service"
```

Preview without writing anything:

```sh
spark service --dry-run
```

Write somewhere other than the current directory:

```sh
spark service -o ~/code
```

## 4. Undo

Changed your mind? Reverse the most recent generation:

```sh
spark undo
```

Files Spark created are deleted; files it overwrote are restored.

## Where to go next

- [Template format](templates/format.md) — the full `[info]`, `[options]`,
  `[[files]]` structure.
- [Placeholders and functions](templates/placeholders.md) — built-in tokens and
  `:read`.
- [Supplying values](templates/values.md) — `--from`, environment, and config
  keywords.
- [Command reference](cli/reference.md) — every flag and subcommand.
