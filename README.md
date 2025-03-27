# Spark: A TOML-Based Project Initializer
![Latest Release](https://img.shields.io/github/v/release/pwnxpl0it/spark?label=Release)  ![Build Status](https://img.shields.io/github/actions/workflow/status/pwnxpl0it/spark/ci.yml?branch=main)  ![License](https://img.shields.io/github/license/pwnxpl0it/spark)  ![Contributions Welcome](https://img.shields.io/badge/contributions-welcome-brightgreen)  

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
- [Creating Templates 📜](#creating-templates-)
- [Dynamic Placeholders and Functions](#dynamic-placeholders-and-functions)
- [Supply/Override Values from CLI (`--from`) 🏗️](#supplyoverride-values-from-cli---from-️)
- [Environment Variables ⚙️](#environment-variables-%EF%B8%8F)
- [Template Options](#template-options)
- [Git Integration 🐙](#git-integration-)
- [Example Templates](#example-templates)
  - [Neovim Plugin](#neovim-plugin)
  - [Jekyll Blog Post](#jekyll-blog-post)
  - [Browser Extension](#browser-extension)
- [JSON Integration](#json-integration)
- [Liquid Templating Support 🧪](#liquid-templating-support-)
- [Automated Template Generation 🚀](#automated-template-generation-)
- [Neovim Plugin](#neovim-plugin-)

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
| use_liquid    | Enable/Disable Liquid templating in the template     | `use_liquid=true` |
| use_json    | Embed JSON in the template     | `use_json='{"id": 1, "name": "John"}'` |


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
/```

## Usage

```python
import {{$PROJECTNAME}}

print({{$PROJECTNAME}}.__version__)
/```

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

You can use json to replace placeholders in your template, spark will automatically load values from a json file and replace them automatically

Spark uses JSON Query language to load values from json nodes.

Here is an example:

```json
{
	"user": {
		"id": "12345",
		"name": "John Doe",
		"email": "john.doe@example.com"
	},
	"status": ["200 OK"]
}
```

Example template:

```toml
[[files]]
path="test"
content="""
User ID: {{$.user.id}}
User Name: {{$.user.name}}
User Email: {{$.user.email}}
Response Status: {{$.status[0]}}
"""
```

```sh
$ spark template --json test.json
```

Output:

```console
$ cat test

User ID: 12345
User Name: John Doe
User Email: john.doe@example.com
Response Status: 200 OK
```

> [!NOTE]
> Although this is a cool feature to automate user inputs, It comes with performance costs
> [Why?](https://github.com/onelson/jq-rs?tab=readme-ov-file#a-note-on-performance)

## Liquid Templating Support 🧪

Spark now supports [Liquid](https://shopify.github.io/liquid/) templating alongside its own custom syntax. This allows you to benefit from Liquid's logic (loops, conditionals) while continuing to use `spark`'s powerful keyword replacement.

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

- *Spark* replaces `{{$file:read}}` with user input.
- Liquid handles loops and string manipulation.

#### **Result:**
```
Example! 1 ff.html
Example! 2 ff.html
Example! 3 ff.html
Example! 4 ff.html
Example! 5 ff.html
```

With this integration, you can create dynamic and flexible templates that combine the strengths of both `spark` and Liquid.

> [!TIP]
> Liquid is enabled by default in templates. To disable it, set `use_liquid=false` in the template options.
> or use `--no-liquid` flag when running `spark`

> [!IMPORTANT]
> When using Spark keywords inside Liquid templates, wrap them in double curly braces like this:
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
Spark finds them stored in $HOME/.config/spark/config.toml Or the config path you specified using -c/--config option 🦀

```toml
[Keywords]
AUTHOR = "Mohamed Tarek"
USERNAME = "@pwnxpl0it"
GITHUB = "https://github.com/pwnxpl0it"
#etc .....
```

## 👾 Neovim plugin (spark.nvim) 
I wrote a neovim plugin that makes it a way easier, Check it out [spark.nvim](https://www.github.com/pwnxpl0it/spark.nvim).
