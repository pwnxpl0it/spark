# Liquid templating

Spark supports [Liquid][liquid] alongside its own placeholders — loops, filters,
and conditionals for templates that need more than substitution.

Liquid is **off by default**, because its `{{ }}` syntax collides with files such
as GitHub Actions workflows, Jinja, Vue, or Handlebars templates. Turn it on per
template with `use_liquid = true` in `[options]`, or per run with `--liquid`.
`--no-liquid` forces it off.

## Processing order

For each file Spark always runs, in order:

1. **Resolve** functions (`:read`) and JSON paths (`{{$.…}}`).
2. **Replace** `{{$…}}` keywords.
3. **Render** Liquid (when enabled).

So keyword and JSON values are available to Liquid filters and tags.

```toml
[[files]]
path = "output.txt"
content = """
{% for i in (1..5) %}
Example! {{ i }} {{ "{{$file:read}}" | append: ".html" }}
{% endfor %}
"""
```

Spark resolves and replaces `{{$file:read}}` first; Liquid then handles the loop
and the filter. With `file = ff`:

```text
Example! 1 ff.html
Example! 2 ff.html
Example! 3 ff.html
Example! 4 ff.html
Example! 5 ff.html
```

## Mixing Spark keywords with Liquid

Because both use `{{ }}`, wrap a Spark keyword in quotes so Liquid sees a string
until after replacement:

```liquid
{{ "{{$PLACEHOLDER}}" | capitalize }}
```

Escaped tokens (`{{$$X}}`) need the same wrapping, or Liquid turned off.

## Template metadata in Liquid

The template's own `[info]` and `[options]` values are injected as Liquid
globals, so you can read and branch on them directly — these are real Liquid
variables, not Spark keywords, so no wrapping is needed:

| Variable | From |
|---|---|
| `info.name`, `info.author`, `info.description` | `[info]` |
| `options.git` (bool) | `[options] git` |
| `options.use_liquid` (bool) | effective `use_liquid` |

```toml
[info]
name = "demo"

[options]
git = true
use_liquid = true

[[files]]
path = "CONTRIBUTING.md"
content = """
# Contributing to {{ info.name | capitalize }}
{% if options.git %}Run `git clone` to get started.{% endif %}
"""
```

[liquid]: https://shopify.github.io/liquid/
