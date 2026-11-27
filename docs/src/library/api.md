# Rust API

Spark is a library crate as well as a CLI. You can embed the template engine to
render in memory, scaffold files, and handle typed errors without shelling out.

## Add the dependency

```toml
[dependencies]
spark = { git = "https://github.com/pwnxpl0it/spark" }
serde_json = "1"   # for the json!() macro used below
```

## Core types

| Type | Description |
|---|---|
| `Template` | A parsed template (files + options + info). |
| `Context` | Variables, JSON data, and interactivity mode. |
| `RenderedFile` | Output of `render` — evaluated path and content. |
| `Error` | Typed error enum (`InvalidPath`, `MissingVariable`, `Io`, …). |
| `Result<T>` | `std::result::Result<T, spark::Error>`. |

## Parse a template

```rust
use spark::Template;

// From a string
let template = Template::from_str(r#"
[[files]]
path = "{{$DIR}}/README.md"
content = "# {{$NAME}}"
"#)?;

// From a file. Rust does not expand `~`, so build the path explicitly.
let home = std::env::var("HOME").expect("HOME not set");
let template = Template::from_file(format!("{home}/.config/spark/templates/rust.toml"))?;
```

## Build a template in code

```rust
use spark::{Template, File, Information};

let template = Template::builder()
    .with_info(Information::new(Some("My App".into()), Some("me".into()), None))
    .with_file(File::create("src/main.rs", "fn main() {}"))
    .with_file(File::create("README.md", "# {{$NAME}}"));
```

## Render in memory

`Template::render` evaluates every placeholder and returns `Vec<RenderedFile>`
without touching the filesystem — useful for tests and previews.

```rust
use spark::{Template, Context};

let template = Template::from_str(r#"
[[files]]
path = "{{$SLUG}}/main.rs"
content = "// generated for {{$AUTHOR}}"
"#)?;

let ctx = Context::new()
    .with_var("SLUG",   "my_crate")
    .with_var("AUTHOR", "Alice")
    .non_interactive();            // never blocks on stdin

let files = template.render(&ctx)?;
assert_eq!(files[0].path,    "my_crate/main.rs");
assert_eq!(files[0].content, "// generated for Alice");
```

## Supply variables from a map or array

`Context::with_vars` accepts any iterator of pairs, including a `HashMap`. Bare
keys like `"NAME"` are formatted as `{{$NAME}}` automatically.

```rust
use spark::{Template, Context};

let ctx = Context::new()
    .with_vars([("NAME", "alice"), ("ROLE", "admin")])
    .non_interactive();
```

Prefer `with_vars` over `Context::from(hashmap)`: `From` copies keys as-is, so
they must already be in `{{$NAME}}` form.

## Write files to disk

`Template::extract_with_context` renders **and** writes — to the filesystem,
`stdout://`, `stderr://`, or `clipboard://`.

```rust
use spark::{Template, Context};

let template = Template::from_file("templates/backend.toml")?;
let ctx = Context::new().with_var("NAME", "my_service").non_interactive();

let written = template.extract_with_context(&ctx)?;
for f in &written {
    println!("wrote → {}", f.path);
}
```

On a headless system a `clipboard://` write fails with `Error::OutputWrite`
rather than panicking. Build with `--no-default-features` to drop clipboard
support entirely.

## Supply JSON data

```rust
use spark::{Template, Context};
use serde_json::json;

let template = Template::from_str(r#"
[[files]]
path = "{{$.user.name}}/profile.txt"
content = "Email: {{$.user.email}}"
"#)?;

let ctx = Context::new()
    .with_json(json!({ "user": { "name": "alice", "email": "alice@example.com" } }))
    .non_interactive();

let files = template.render(&ctx)?;
assert_eq!(files[0].path, "alice/profile.txt");
```

## Handle errors

```rust
use spark::{Template, Context, Error, File};

let template = Template::builder()
    .with_file(File::create("{{$PROJECTNAME}}/main.rs", "fn main() {}"));

let ctx = Context::new().non_interactive(); // PROJECTNAME not supplied

match template.render(&ctx) {
    Err(Error::MissingVariable(name)) => eprintln!("missing: {name}"),
    Err(e) => eprintln!("error: {e}"),
    Ok(files) => { /* ... */ }
}
```

## Full reference

```sh
cargo doc --open
```
