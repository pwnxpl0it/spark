# Supplying values

A placeholder can get its value from several sources. When the same name is
defined in more than one, the higher-priority source wins. From lowest to
highest:

1. Built-ins (`HOME`, `NOW`, `YYYY`, …) and **every environment variable**
   (with `.env` loaded first).
2. The `[Keywords]` table in your config file.
3. `--from key=value` pairs on the command line.

JSON data (`--json` or `[options.json_data]`) is separate: it feeds `{{$.path}}`
lookups rather than the keyword map. See [JSON data](json.md).

## From the command line: `--from`

Pass values directly to skip interactive prompts:

```sh
spark template --from "name=spark, author=pwnxpl0it, hackerman=yes"
```

Given this template:

```toml
[[files]]
path = "README.md"
content = """
# {{$name}}

Created by {{$author}}

Hackerman mode: {{$hackerman}}
"""
```

`--from "name=MyProject, author=JohnDoe, hackerman=no"` produces:

```md
# MyProject

Created by JohnDoe

Hackerman mode: no
```

`--from` can be repeated, and a comma stays inside a value when the text after it
has no `=`:

```sh
spark template --from "tags=rust,cli" --from "author=JohnDoe"
```

## Environment variables and `.env`

Any environment variable is usable as a placeholder, and Spark loads a `.env`
file from the working directory first.

```env
DB_HOST=localhost
DB_PORT=5432
```

```toml
[[files]]
path = "config.py"
content = """
DB_HOST = "{{$DB_HOST}}"
DB_PORT = "{{$DB_PORT}}"
"""
```

## Config keywords

Define your own reusable values in `[Keywords]` in the config file
(`~/.config/spark/config.toml`, or the path given with `-c`/`--config`):

```toml
[Keywords]
AUTHOR = "Mohamed Tarek"
USERNAME = "@pwnxpl0it"
GITHUB = "https://github.com/pwnxpl0it"
```

Every template can then use `{{$AUTHOR}}`, `{{$GITHUB}}`, and so on.

## Unresolved placeholders

A `{{$NAME}}` with no value from any source is replaced with an empty string and
a warning is printed. Pass `--strict` to abort instead:

```sh
spark template --strict
```

Library callers choose the behaviour with
`Context::on_missing(MissingPolicy::{Empty, Keep, Error})`.
