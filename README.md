# Spark: A TOML-Based Project Initializer
 [![DeepWiki](https://img.shields.io/badge/DeepWiki-pwnxpl0it%2Fspark-blue.svg?logo=data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAACwAAAAyCAYAAAAnWDnqAAAAAXNSR0IArs4c6QAAA05JREFUaEPtmUtyEzEQhtWTQyQLHNak2AB7ZnyXZMEjXMGeK/AIi+QuHrMnbChYY7MIh8g01fJoopFb0uhhEqqcbWTp06/uv1saEDv4O3n3dV60RfP947Mm9/SQc0ICFQgzfc4CYZoTPAswgSJCCUJUnAAoRHOAUOcATwbmVLWdGoH//PB8mnKqScAhsD0kYP3j/Yt5LPQe2KvcXmGvRHcDnpxfL2zOYJ1mFwrryWTz0advv1Ut4CJgf5uhDuDj5eUcAUoahrdY/56ebRWeraTjMt/00Sh3UDtjgHtQNHwcRGOC98BJEAEymycmYcWwOprTgcB6VZ5JK5TAJ+fXGLBm3FDAmn6oPPjR4rKCAoJCal2eAiQp2x0vxTPB3ALO2CRkwmDy5WohzBDwSEFKRwPbknEggCPB/imwrycgxX2NzoMCHhPkDwqYMr9tRcP5qNrMZHkVnOjRMWwLCcr8ohBVb1OMjxLwGCvjTikrsBOiA6fNyCrm8V1rP93iVPpwaE+gO0SsWmPiXB+jikdf6SizrT5qKasx5j8ABbHpFTx+vFXp9EnYQmLx02h1QTTrl6eDqxLnGjporxl3NL3agEvXdT0WmEost648sQOYAeJS9Q7bfUVoMGnjo4AZdUMQku50McDcMWcBPvr0SzbTAFDfvJqwLzgxwATnCgnp4wDl6Aa+Ax283gghmj+vj7feE2KBBRMW3FzOpLOADl0Isb5587h/U4gGvkt5v60Z1VLG8BhYjbzRwyQZemwAd6cCR5/XFWLYZRIMpX39AR0tjaGGiGzLVyhse5C9RKC6ai42ppWPKiBagOvaYk8lO7DajerabOZP46Lby5wKjw1HCRx7p9sVMOWGzb/vA1hwiWc6jm3MvQDTogQkiqIhJV0nBQBTU+3okKCFDy9WwferkHjtxib7t3xIUQtHxnIwtx4mpg26/HfwVNVDb4oI9RHmx5WGelRVlrtiw43zboCLaxv46AZeB3IlTkwouebTr1y2NjSpHz68WNFjHvupy3q8TFn3Hos2IAk4Ju5dCo8B3wP7VPr/FGaKiG+T+v+TQqIrOqMTL1VdWV1DdmcbO8KXBz6esmYWYKPwDL5b5FA1a0hwapHiom0r/cKaoqr+27/XcrS5UwSMbQAAAABJRU5ErkJggg==)](https://deepwiki.com/pwnxpl0it/spark)
![Latest Release](https://img.shields.io/github/v/release/pwnxpl0it/spark?label=Release)  ![Build Status](https://img.shields.io/github/actions/workflow/status/pwnxpl0it/spark/ci.yml?branch=main)  ![License](https://img.shields.io/github/license/pwnxpl0it/spark)  ![Contributions Welcome](https://img.shields.io/badge/contributions-welcome-brightgreen) 
<br>
Spark is a powerful and flexible project initializer designed to simplify your workflow. Using a TOML-based configuration, Spark allows you to quickly create project directories and files based on predefined templates. Whether you're a developer looking for consistency or speed, Spark has you covered.

**Features:**
- **TOML-based templates** for structured project creation.
- Support for **environment variables** and **dotenv** files.
- **Dynamic placeholders** and **custom functions** for template customization.
- **Constant values from configuration**
- **JSON Support** to automate user input 
- Integration with **Git** for version control setup.
- **Liquid templating support** for advanced customization.
- **And more..**

> [!WARNING]
> This project was previously named `idkmng`. Some issues and references may still use the old name.

---

## Table of Contents

- [Why Spark? 🧠](#why-spark-)
- [Installation](#installation)
- [Using Spark as a Library 📦](#using-spark-as-a-library-)
- [Creating Templates 📜](#creating-templates-)
- [Dynamic Placeholders and Functions](#dynamic-placeholders-and-functions)
- [Supply/Override Values from CLI (`--from`) 🏗️](#supplyoverride-values-from-cli---from-️)
- [Environment Variables ⚙️](#environment-variables-%EF%B8%8F)
- [Template Options](#template-options)
- [Output Targets 🎯](#output-targets-)
- [Git Integration 🐙](#git-integration-)
- [Example Templates](#example-templates)
  - [Neovim Plugin](#neovim-plugin)
  - [Jekyll Blog Post](#jekyll-blog-post)
  - [Browser Extension](#browser-extension)
- [JSON Integration](#json-integration)
- [Liquid Templating Support 🧪](#liquid-templating-support-)
- [Automated Template Generation 🚀](#automated-template-generation-)
- [Config Keywords ⚙️](#config-keywords-%EF%B8%8F)
- [Development](#development)
- [Neovim plugin (spark.nvim)](#-neovim-plugin-sparknvim)

---

## Why Spark? 🧠

Creating projects often involves repetitive tasks, such as setting up directories and boilerplate files. Spark streamlines this process by allowing you to define templates in TOML. For instance, to create a browser extension, simply run:

```sh
spark browser_extension
```

And voilà! Your project is ready for the first commit. Spark's flexibility means you can define multiple templates for various use cases, ensuring your projects always start with the structure you need.

---

## Installation

### Via Cargo (Requires Rust 🦀)
Install Spark directly from the GitHub repository:

```sh
cargo install --git https://github.com/pwnxpl0it/spark
```

### Precompiled Binaries
Download a precompiled binary from the [Releases](https://github.com/pwnxpl0it/spark/releases) page:

```sh
sudo tar -xzf spark-<VERSION>.tar.gz -C /usr/local/bin
```

Replace `<VERSION>` with the desired release version.

Verify installation by running:

```sh
spark --version
```

---

## Using Spark as a Library 📦

Spark is not only a CLI tool — it is a first-class Rust library crate. You can embed Spark's template engine directly into your own applications to perform **in-memory rendering**, **file scaffolding**, and **typed error handling**, all without shelling out.

### Add to your project

```toml
[dependencies]
spark = { git = "https://github.com/pwnxpl0it/spark" }
serde_json = "1"   # required for the json!() macro used in JSON examples
```

### Core types

| Type | Description |
|---|---|
| `Template` | A parsed TOML template (files + options + info) |
| `Context` | Variables, JSON data, and interactivity mode |
| `RenderedFile` | Output of `Template::render` — evaluated path + content |
| `Error` | Typed error enum (`InvalidPath`, `MissingVariable`, `Io`, …) |
| `Result<T>` | `std::result::Result<T, spark::Error>` |

### Parse a template

```rust
use spark::Template;

// From a &str
let template = Template::from_str(r#"
[[files]]
path = "{{$DIR}}/README.md"
content = "# {{$NAME}}"
"#)?;

// From a file path – note: Rust does not expand `~`; construct the path explicitly
let home = std::env::var("HOME").expect("HOME not set");
let template = Template::from_file(format!("{home}/.spark/templates/rust.toml"))?;
```

### Build a template programmatically

```rust
use spark::{Template, File, Information};

let template = Template::builder()
    .with_info(Information::new(
        Some("My App".into()),
        Some("me".into()),
        None,
    ))
    .with_file(File::create("src/main.rs", "fn main() {}"))
    .with_file(File::create("README.md", "# {{$NAME}}"));
```

### Render in memory (no I/O)

`Template::render` evaluates all placeholders and returns a `Vec<RenderedFile>` without touching the filesystem. Perfect for testing or preview.

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

### Write files to disk (with output targets)

`Template::extract_with_context` renders **and** writes to disk, `stdout://`, `stderr://`, or `clipboard://`.

```rust
use spark::{Template, Context};

let template = Template::from_file("templates/backend.toml")?;

let ctx = Context::new()
    .with_var("NAME", "my_service")
    .non_interactive();

let written = template.extract_with_context(&ctx)?;
for f in &written {
    println!("wrote → {}", f.path);
}
```

### Supply JSON data programmatically

```rust
use spark::{Template, Context};
use serde_json::json;

let template = Template::from_str(r#"
[[files]]
path = "{{$.user.name}}/profile.txt"
content = "Email: {{$.user.email}}"
"#)?;

let ctx = Context::new()
    .with_json(json!({
        "user": { "name": "alice", "email": "alice@example.com" }
    }))
    .non_interactive();

let files = template.render(&ctx)?;
assert_eq!(files[0].path, "alice/profile.txt");
```

### Error handling

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

### API reference

Full rustdoc is available by running:

```sh
cargo doc --open
```

---

## Creating Templates 📜


To create a new template, run:

```sh
spark new
```

This will generate a basic template file in `~/.config/spark/templates/<TEMPLATE_NAME>.toml`. 

The template structure is as follows:

```toml
[info]
name = "Template Name"
description = "Template Description"
author = "Your Name"

[[files]]
path = "file1.txt"
content = """
Content of file 1
"""

[[files]]
path = "file2.txt"
content = """
Content of file 2
"""
```
> [!TIP]
> **Tip**: The `[info]` section is optional and can be removed.

### Placeholder Format
Use `{{$PLACEHOLDER}}` for dynamic content replacement. Common placeholders include:


| placeholder   | Value     | Example          |
|--------------- | ---------------  | ---------------  |
| PROJECTNAME   | Asks for project name |                   |
| CURRENTDIR    | Current directory | pwd=/foo/bar => `bar`|
| HOME          | Home directory    | `/home/user/`    |
| YYYY    | Current Year in YYYY format| 2024    |
| YY | Current Year in YY format| 24    |
| MM | Current Month | 2 |
| DD | Current Day | 24 |
| NOW | Current date and time | `2024-02-23 22:22:38.151417626 +00:00` |
| NOW_UTC | Current date and time in UTC | `2024-02-23 22:21:17.897444668 UTC` |



## Dynamic Placeholders and Functions

Enhance templates with functions for additional customization. Functions follow the format `{{$PLACEHOLDER:FUNCTION}}`.

### Supported Functions
| Function | Description                         | Example                |
|----------|-------------------------------------|------------------------|
| `read`   | Prompts for user input              | `{{$VAR:read}}`        |

Example template snippet:

```toml
[[files]]
path = "example.txt"
content = """
User input: {{$USER_INPUT:read}}
"""
```

---
### **Supply/Override Values from CLI (`--from`)** 🏗️  

You can now pass predefined values directly from the command line using the `--from` flag. This allows you to override placeholders and avoid interactive prompts.

#### **Usage:**
```sh
spark template --from="name=spark, author=pwnxpl0it, hackerman=yes"
```

This expands the keywords hashmap and skips calling functions like `:read`, meaning users can directly provide values from the command line.

#### **Example Template:**
```toml
[[files]]
path = "README.md"
content = """
# {{$name}}

Created by {{$author}}

Hackerman mode: {{$hackerman}}
"""
```

#### **Running Spark:**
```sh
spark /path/to/template --from="name=MyProject, author=JohnDoe, hackerman=no"
```

#### **Generated File (`README.md`):**
```md
# MyProject

Created by JohnDoe

Hackerman mode: no
```

With this feature, you can fully automate project creation without interactive prompts! 🚀

---

## Environment Variables ⚙️

Spark supports placeholders that map to environment variables. You can also use `.env` files for placeholder substitution.

Example `.env` file:

```env
DB_HOST=localhost
DB_PORT=5432
```

Example template:

```toml
[[files]]
path = "config.py"
content = """
DB_HOST = "{{$DB_HOST}}"
DB_PORT = "{{$DB_PORT}}"
"""
```

Generated file:

```python
DB_HOST = "localhost"
DB_PORT = "5432"
```

---

### Template Options

Template options in spark provide a way to customize the project setup by allowing predefined variables or settings within the template. These options are defined in the TOML configuration file of the template and can control various aspects of the template generation process.

| Option   | Description    | Example  |
|--------------- | --------------- | ---------------  |
| git   | Initialize Git repository in the project directory   | `git=true` |
| project_root    | Set the project name to a constant value or ask for user input  | `project_root="new_project"`, `project_root="{{$PROJECTNAME}}"` |
| use_liquid    | Enable/Disable Liquid templating in the template (enabled by default)     | `use_liquid=true` |
| json_data    | Embed JSON in the template for `{{$.…}}` placeholders     | See [JSON Integration](#json-integration) |


## Output Targets 🎯

By default, `[[files]].path` is a filesystem path. You can prefix the path with a protocol scheme to redirect rendered output to a different sink.

| Scheme        | Behaviour                                      |
|---------------|------------------------------------------------|
| *(no scheme)* | Write to the filesystem (unchanged behaviour)  |
| `file://path` | Write to `path` on the filesystem (explicit)   |
| `stdout://`   | Write rendered content to **stdout**           |
| `stderr://`   | Write rendered content to **stderr**           |
| `clipboard://` | Copy rendered content to the system clipboard  |

Unrecognized schemes (e.g. `ftp://`) are not treated as protocol targets and fall back to plain filesystem output.

### Examples

Print a rendered message to stdout instead of creating a file:

```toml
[[files]]
path = "stdout://"
content = "Hello {{$NAME}}!"
```

Send a warning to stderr:

```toml
[[files]]
path = "stderr://"
content = "warning: {{$MESSAGE}}"
```

Use `file://` for an explicit filesystem path:

```toml
[[files]]
path = "file://src/main.rs"
content = """
fn main() {}
"""
```

Copy rendered content to the system clipboard:

```toml
[[files]]
path = "clipboard://"
content = """
{
  "api_key": "{{$API_KEY}}",
  "endpoint": "{{$ENDPOINT}}"
}
"""
```

You can mix targets freely in one template:

```toml
[[files]]
path = "src/main.rs"          # normal file
content = "fn main() {}"

[[files]]
path = "stdout://"            # also print a summary to stdout
content = "✅ Generated {{$PROJECTNAME}}"
```

> [!NOTE]
> Windows drive-letter paths such as `C:\Users\foo` are **never** mis-parsed as
> protocol URIs — the scheme detector requires more than one character before the
> colon.
>
> [!NOTE]
> All Spark templating (keyword replacement, JSON paths, Liquid) is applied
> before the output is dispatched to the target.  `stdout://{{$TARGET}}` is not
> supported as a dynamic target selector; the protocol prefix must be a literal
> string after all placeholder replacements.


## Git Integration 🐙

Initialize a Git repository during project creation:

```sh
spark /path/to/template --git
```

Alternatively, include Git setup in the template:

```toml
[options]
git = true
#project_root = "my_project"
project_root="{{$PROJECTNAME}}" # will prompt for the project name but you can set this to constant value
```

---

## Example Templates

### Example Templates
Here are a few examples:

<details>
  <summary>Neovim Plugin [Click to expand]</summary>

```toml
[options]
git=true
project_root="{{$PROJECTNAME}}"
 
[info]
name = "Neovim Plugin"
description = "A template for nvim plugin"
author = "Mohamed Tarek @pwnxpl0it"

[[files]]
path="{{$PROJECTNAME}}/lua/{{$PROJECTNAME}}/init.lua"
content="""
local M = {}

M.config = {}

M.setup = function ()
   if config ~= nil then
        M.config = config
    end

end

return M
"""

[[files]]
path="{{$PROJECTNAME}}/plugin/init.lua"
content="""
require("{{$PROJECTNAME}}")
"""
```

</details>


<details>
<summary>Python package [Click to expand]</summary>

```toml
[info]
name = "Python Package"
description = "A template for creating a Python package."
author = "Mohamed Tarek @pwnxpl0it"

[options]
git = true
project_root = "{{$PROJECTNAME}}"
use_liquid = true

[[files]]
path = "{{$PROJECTNAME}}/{{$PROJECTNAME}}/__init__.py"
content = """
\"\"\"
{{$PROJECTNAME}}: {{$DESCRIPTION:read}}
\"\"\"

__version__ = "0.1.0"
"""

[[files]]
path = "{{$PROJECTNAME}}/setup.py"
content = """
from setuptools import setup, find_packages

setup(
    name="{{$PROJECTNAME}}",
    version="0.1.0",
    author="{{$AUTHOR:read}}",
    description="{{$DESCRIPTION:read}}",
    packages=find_packages(),
    install_requires=[],
)
"""

[[files]]
path = "{{$PROJECTNAME}}/README.md"
content = """
# {{ "{{$PROJECTNAME}}" | capitalize }}

{{$DESCRIPTION}}

## Installation

```sh
pip install {{$PROJECTNAME}}
```

## Usage

```python
import {{$PROJECTNAME}}

print({{$PROJECTNAME}}.__version__)
```

## License
This project is licensed under the MIT License.
"""

[[files]]
path = "{{$PROJECTNAME}}/.gitignore"
content = """
# Ignore Python build files
__pycache__/
*.pyc
*.pyo
*.pyd
*.so
*.egg-info/
dist/
build/
"""

[[files]]
path = "{{$PROJECTNAME}}/tests/test_{{$PROJECTNAME}}.py"
content = """
import unittest
import {{$PROJECTNAME}}

class Test{{ "{{$PROJECTNAME}}" | capitalize }}(unittest.TestCase):
    def test_version(self):
        self.assertEqual({{$PROJECTNAME}}.__version__, "0.1.0")

if __name__ == "__main__":
    unittest.main()
"""
```

</details>

<details>
    <summary>Jekyll new blogpost [Click to expand]</summary>

I use this template to create a new post in my blog directly from CLI,This one here uses more keywords and includes a private BLOGPATH placeholder that it's value is loaded from config file.

```toml
[info]
name = "new_post"
description = "New jekyll post"
author = "Mohamed Tarek @pwnxpl0it"

[[files]]
path="{{$BLOGPATH}}/_posts/{{$YYYY}}-{{$MM}}-{{$DD}}-{{$blogtitle:read}}.markdown"
content="""
---
layout: post
title: "{{$blogtitle}}"
date: {{$NOW_UTC}}
tags: {{$Tags:read}}
---

"""

```

</details>

<details>
    <summary>Browser (Chrome) Extension [Click to expand]</summary>
This one is just for creating a really BASIC chrome extension.

	
```toml
[options]
git=true
project_root="{{$PROJECTNAME}}"

[info]
name = "browser_extension"
description = "A Template for creating a browser extension"
author = "Mohamed Tarek @pwnxpl0it"
refrence= "https://developer.chrome.com/docs/extensions/mv3/manifest/"

[[files]]
path="{{$PROJECTNAME}}/manifest.json"
content="""
{
  "manifest_version": 3,
  "name":"{{$PROJECTNAME}}",
  "version": "1.0.1",
  "content_scripts":[
    {
     "matches":["<all_urls>"],
     "js":["content.js"]
    }
  ]
}
"""

[[files]]
path="{{$PROJECTNAME}}/content.js"
content="""
console.log("Hello world!")
"""

```

 Info section can have any additional values, it won't get printed but maybe usefull when sharing the template or just as a reference for docs like I did here
 
</details>



---

## JSON Integration

You can drive placeholders from JSON — either a file via `--json`, or data embedded in the template under `[options.json_data]`.

Spark uses [jq](https://jqlang.github.io/jq/)-style paths. Placeholders look like `{{$.user.name}}` and work in both **file paths** and **content**.

### From a JSON file

```json
{
	"user": {
		"id": "12345",
		"name": "John Doe",
		"email": "john.doe@example.com"
	},
	"project": {
		"slug": "demo_app"
	},
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
$ spark template --json test.json
```

### Embedded in the template

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

> [!NOTE]
> JSON lookup uses [`jaq`](https://github.com/01mf02/jaq) (a fast, pure-Rust implementation of `jq`), requiring no external C libraries.

## Liquid Templating Support 🧪

Spark supports [Liquid](https://shopify.github.io/liquid/) alongside its own placeholders (loops, filters, conditionals, etc.).

### Processing order

For each file, Spark always runs in this order:

1. **Resolve** functions (`:read`) and JSON paths (`{{$.…}}`) in path and content  
2. **Replace** `{{$…}}` keywords  
3. **Render** Liquid (when enabled)

So keyword/JSON values are available to Liquid filters and tags.

#### **Example:**
```toml
[[files]]
path = "output.txt"
content = """
{% for i in (1..5) %}
Example! {{ i }} {{ "{{$file:read}}" | append: ".html" }}
{% endfor %}
"""
```

- Spark resolves/replaces `{{$file:read}}` first.
- Liquid then handles the loop and filters.

#### **Result:**
```
Example! 1 ff.html
Example! 2 ff.html
Example! 3 ff.html
Example! 4 ff.html
Example! 5 ff.html
```

> [!TIP]
> Liquid is enabled by default. Disable with `use_liquid=false` in `[options]`, or `--no-liquid` on the CLI.

---

> [!IMPORTANT]
> When using Spark keywords inside Liquid, wrap them so Liquid sees a string until after replacement:
>   ```liquid
>     {{ "{{$PLACEHOLDER}}" | capitalize }}
>   ```

## Automated Template generation 🚀
Also there is one more time saving way! if you have some files in `/foo/bar/` you can just run `spark init` and it will create a template for you with directory name `bar.toml` and it will have all your files in it! 🌸

```console
$ tree
.
├── lua
│   └── test123
│       └── init.lua
└── plugin
    └── init.lua

4 directories, 2 files

$ spark init
Creating Template: test123.toml
```

```console
$ cat test123.toml

[[files]]
path = 'plugin/init.lua'
content = '''
require("test123")
'''

[[files]]
path = 'lua/test123/init.lua'
content = '''
local M = {}

M.config = {}

M.setup = function ()
   if config ~= nil then
        M.config = config
    end

end

return M
'''

```

## Config Keywords ⚙️
You can have your own Keywords for spark to replace with desired values!
Spark finds them stored in `$HOME/.config/spark/config.toml` or the config path you specified using `-c`/`--config`.

```toml
[Keywords]
AUTHOR = "Mohamed Tarek"
USERNAME = "@pwnxpl0it"
GITHUB = "https://github.com/pwnxpl0it"
#etc .....
```

## Development

```sh
cargo build
cargo test
cargo run -- [TEMPLATE] [OPTIONS]
```

Unit tests cover keywords, functions, utils, templates (including JSON paths and Liquid ordering), options, config, CLI args, and the `--from` flag (pre-populating keywords to skip interactive prompts).

## 👾 Neovim plugin (spark.nvim) 
I wrote a neovim plugin that makes it a way easier, Check it out [spark.nvim](https://www.github.com/pwnxpl0it/spark.nvim).
