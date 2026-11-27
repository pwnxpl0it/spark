# Example templates

A few complete templates to adapt. Each is a single `.toml` file you can drop in
your templates directory.

## Neovim plugin

```toml
[options]
git = true
project_root = "{{$PROJECTNAME}}"

[info]
name = "Neovim Plugin"
description = "A template for a nvim plugin"
author = "Mohamed Tarek @pwnxpl0it"

[[files]]
path = "{{$PROJECTNAME}}/lua/{{$PROJECTNAME}}/init.lua"
content = """
local M = {}

M.config = {}

M.setup = function(config)
    if config ~= nil then
        M.config = config
    end
end

return M
"""

[[files]]
path = "{{$PROJECTNAME}}/plugin/init.lua"
content = """
require("{{$PROJECTNAME}}")
"""
```

## Python package

Uses `:read` prompts and Liquid filters.

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
"""

[[files]]
path = "{{$PROJECTNAME}}/.gitignore"
content = """
__pycache__/
*.py[cod]
*.egg-info/
dist/
build/
"""
```

## Jekyll blog post

Writes a new post using date built-ins and a private `BLOGPATH` loaded from your
[config keywords](../templates/values.md#config-keywords).

```toml
[info]
name = "new_post"
description = "New Jekyll post"
author = "Mohamed Tarek @pwnxpl0it"

[[files]]
path = "{{$BLOGPATH}}/_posts/{{$YYYY}}-{{$MM}}-{{$DD}}-{{$blogtitle:read}}.markdown"
content = """
---
layout: post
title: "{{$blogtitle}}"
date: {{$NOW_UTC}}
tags: {{$Tags:read}}
---

"""
```

## Browser extension

```toml
[options]
git = true
project_root = "{{$PROJECTNAME}}"

[info]
name = "browser_extension"
description = "A template for creating a browser extension"
author = "Mohamed Tarek @pwnxpl0it"

[[files]]
path = "{{$PROJECTNAME}}/manifest.json"
content = """
{
  "manifest_version": 3,
  "name": "{{$PROJECTNAME}}",
  "version": "1.0.1",
  "content_scripts": [
    { "matches": ["<all_urls>"], "js": ["content.js"] }
  ]
}
"""

[[files]]
path = "{{$PROJECTNAME}}/content.js"
content = """
console.log("Hello world!")
"""
```

The `[info]` section can hold any extra keys (such as a `reference` URL); they
are never written out, but are handy when sharing a template.
