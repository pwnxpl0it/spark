# Git integration

Spark can initialize a Git repository as part of generation.

Per run, with `--git`:

```sh
spark /path/to/template --git
```

Or set it in the template so it always happens:

```toml
[options]
git = true
project_root = "{{$PROJECTNAME}}"   # prompts for the name; a constant also works
```

The repository is initialized inside the template's `project_root` (the
generated project directory). `--git` only sets `project_root` to
`{{$PROJECTNAME}}` when the template leaves it empty.
