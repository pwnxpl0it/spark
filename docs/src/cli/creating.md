# Creating templates

You can write a template by hand (see [Template format](../templates/format.md)),
or let Spark build one from files you already have.

## From the current directory

Run `spark init` in a directory and Spark captures every file into a template
named after the directory:

```console
$ tree
.
├── lua
│   └── test123
│       └── init.lua
└── plugin
    └── init.lua

$ spark init
Creating Template: test123.toml
```

The generated `test123.toml` contains each file's path and content:

```toml
[[files]]
path = 'plugin/init.lua'
content = '''
require("test123")
'''

[[files]]
path = 'lua/test123/init.lua'
content = '''
local M = {}
-- ...
'''
```

Any placeholder-looking text in the captured files is escaped (`{{$$…}}`) so the
generated template reproduces them verbatim.

## Presetting `[options]`

`spark init` can embed an `[options]` table so you don't have to hand-edit it:

| Flag | Effect |
|---|---|
| `--git` | sets `git = true` |
| `--liquid` | sets `use_liquid = true` |
| `--project-root VALUE` | sets `project_root` (e.g. `--project-root '{{$PROJECTNAME}}'`) |

```console
$ spark init --git --project-root '{{$PROJECTNAME}}'
Creating Template: test123.toml

$ head -4 test123.toml
[options]
git = true
project_root = "{{$PROJECTNAME}}"
```

A plain `spark init` with no option flags omits the `[options]` table entirely.

## From a GitHub repository

Pass `--from-github <owner/repo>` to build the template from a repository
instead of the current directory. Spark shallow-clones the repo into a temporary
directory using your installed `git` (so your existing auth works for private
repos), captures its files, and writes `<repo>.toml`:

```console
$ spark init --from-github pwnxpl0it/spark
Cloning: https://github.com/pwnxpl0it/spark.git
Creating Template: spark.toml
```

- Add a branch or tag with `@ref`: `spark init --from-github owner/repo@v2.0.1`.
- The repo's `.git` directory is never captured, and its `.gitignore` is honored,
  so build artifacts stay out of the template.
- The `--git`, `--liquid`, and `--project-root` flags apply here too.
