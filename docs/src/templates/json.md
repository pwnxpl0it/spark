# JSON data

Spark can drive placeholders from JSON, using `jq`-style paths. A JSON
placeholder looks like `{{$.user.name}}` and works in both paths and content.

JSON lookups use [`jaq`][jaq], a pure-Rust implementation of `jq`, so there is no
external C dependency.

## From a file: `--json`

```json
{
  "user": {
    "id": "12345",
    "name": "John Doe",
    "email": "john.doe@example.com"
  },
  "project": { "slug": "demo_app" },
  "status": ["200 OK"]
}
```

```toml
[[files]]
path = "{{$.project.slug}}/README.md"
content = """
User ID: {{$.user.id}}
User Name: {{$.user.name}}
User Email: {{$.user.email}}
Response Status: {{$.status[0]}}
"""
```

```sh
spark template --json data.json
```

The `--json` path is resolved relative to where you run the command, before
`--output-dir` takes effect.

## Embedded in the template

Put the data under `[options.json_data]` so the template is self-contained:

```toml
[options]
git = false
use_liquid = false

[options.json_data]
status = ["200 OK"]

[options.json_data.user]
id = "12345"
name = "John Doe"
email = "john.doe@example.com"

[options.json_data.project]
slug = "demo_app"

[[files]]
path = "{{$.project.slug}}/profile.txt"
content = """
Hello {{$.user.name}}
"""
```

[jaq]: https://github.com/01mf02/jaq
