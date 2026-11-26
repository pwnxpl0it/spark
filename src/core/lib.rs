//! # Spark ⚡
//!
//! A fast, minimalist, and extensible scaffolding and templating engine written in Rust.
//!
//! Spark allows you to define declarative templates in TOML, resolve dynamic placeholders,
//! evaluate JSON path queries via `jaq`, run Liquid templating filters, and render outputs
//! in-memory or dispatch them to files, stdout, stderr, or the clipboard.
//!
//! ## Using Spark as a Library
//!
//! Add Spark to your `Cargo.toml`:
//!
//! ```toml
//! [dependencies]
//! spark = "2.0.0"
//! serde_json = "1"   # required for the json!() macro used in JSON examples
//! ```
//!
//! ### Quick Example: In-Memory Rendering
//!
//! ```rust
//! use spark::{Template, Context};
//!
//! // Parse a TOML template
//! let toml_str = r#"
//! [[files]]
//! path = "{{$PROJECT}}/greeting.txt"
//! content = "Hello, {{$USER}}! Welcome to {{$PROJECT}}."
//! "#;
//!
//! let template = Template::from_str(toml_str)?;
//!
//! // Build context with variables
//! let context = Context::new()
//!     .with_var("PROJECT", "my_app")
//!     .with_var("USER", "Alice")
//!     .non_interactive();
//!
//! // Render to memory without creating files on disk
//! let rendered_files = template.render(&context)?;
//!
//! assert_eq!(rendered_files.len(), 1);
//! assert_eq!(rendered_files[0].path, "my_app/greeting.txt");
//! assert_eq!(rendered_files[0].content, "Hello, Alice! Welcome to my_app.");
//! # Ok::<(), spark::Error>(())
//! ```
//!
//! ### JSON Integration with `jaq`
//!
//! ```rust
//! use spark::{Template, Context};
//! use serde_json::json;
//!
//! let toml_str = r#"
//! [[files]]
//! path = "{{$.project.slug}}/profile.txt"
//! content = "Owner: {{$.user.name}} <{{$.user.email}}>"
//! "#;
//!
//! let template = Template::from_str(toml_str)?;
//!
//! let context = Context::new()
//!     .with_json(json!({
//!         "user": { "name": "John Doe", "email": "john@example.com" },
//!         "project": { "slug": "demo_project" }
//!     }))
//!     .non_interactive();
//!
//! let rendered = template.render(&context)?;
//! assert_eq!(rendered[0].path, "demo_project/profile.txt");
//! assert_eq!(rendered[0].content, "Owner: John Doe <john@example.com>");
//! # Ok::<(), spark::Error>(())
//! ```

pub mod context;
pub mod error;
pub mod funcs;
pub mod hooks;
pub mod keywords;
pub mod output_target;
pub mod patch;
pub mod templates;
mod utils;

pub use context::{Context, MissingPolicy};
pub use error::{Error, Result};
pub use hooks::Hooks;
pub use output_target::OutputTarget;
pub use patch::{FileMode, Position};
use serde::{Deserialize, Serialize};
pub use templates::{Options, RenderedFile};

/// Utility container for keyword operations and default variable initializations.
pub struct Keywords {}

/// Template metadata information.
#[derive(Debug, Clone, Default, Deserialize, Serialize, PartialEq, Eq)]
pub struct Information {
    /// Template name.
    pub name: Option<String>,
    /// Template author.
    pub author: Option<String>,
    /// Brief template description.
    pub description: Option<String>,
}

impl Information {
    /// Creates a new `Information` metadata container.
    pub fn new(name: Option<String>, author: Option<String>, description: Option<String>) -> Self {
        Self {
            name,
            author,
            description,
        }
    }
}

/// A file definition entry within a template.
///
/// By default `content` **overwrites** the file at `path`. Set [`File::mode`]
/// (TOML `mode = "append" | "prepend" | "insert" | "block" | "patch"`) to merge
/// the rendered content into an existing file instead; see [`patch`] for the
/// semantics of each mode.
#[derive(Debug, Clone, Default, Deserialize, Serialize, PartialEq, Eq)]
pub struct File {
    /// Target path template (supports placeholders and target URIs).
    pub path: String,
    /// Content template (supports placeholders, functions, and Liquid tags).
    pub content: String,
    /// Write strategy. `None` is equivalent to [`FileMode::Overwrite`].
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub mode: Option<FileMode>,
    /// Anchor line for [`FileMode::Insert`] (supports placeholders).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub anchor: Option<String>,
    /// Insert side for [`FileMode::Insert`]. `None` is equivalent to
    /// [`Position::After`].
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub position: Option<Position>,
    /// Block id for [`FileMode::Block`] (supports placeholders). `None` uses the
    /// default `spark` id.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub marker: Option<String>,
}

/// Represents a complete Spark template with metadata, configuration options, and files.
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct Template {
    /// Optional metadata about the template.
    pub info: Option<Information>,
    /// Execution and engine options.
    pub options: Option<Options>,
    /// Commands to run before and after generation. Never executed by the
    /// library itself; the caller runs them explicitly (the CLI gates this
    /// behind `--run-hooks`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub hooks: Option<Hooks>,
    /// List of file templates to render.
    pub files: Option<Vec<File>>,
}

/// Dynamic function modifiers for template keywords (e.g. `:read`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Fns {
    /// Prompts the user interactively on `stdin` for input. The optional
    /// label (`{{$NAME:read("Project name")}}`) replaces the variable name
    /// in the prompt.
    Read(Option<String>),
    /// Plain variable without function evaluation.
    None,
}

impl File {
    /// Creates a new `File` entry with path and content strings
    /// (overwrite mode).
    pub fn new(path: String, content: String) -> Self {
        Self {
            path,
            content,
            ..Default::default()
        }
    }

    /// Creates a new `File` entry from any types converting into `String`
    /// (overwrite mode).
    pub fn create(path: impl Into<String>, content: impl Into<String>) -> Self {
        Self {
            path: path.into(),
            content: content.into(),
            ..Default::default()
        }
    }

    /// Returns the resolved write strategy, defaulting to
    /// [`FileMode::Overwrite`].
    pub fn mode(&self) -> FileMode {
        self.mode.unwrap_or_default()
    }

    /// Returns the resolved insert position, defaulting to [`Position::After`].
    pub fn position(&self) -> Position {
        self.position.unwrap_or_default()
    }

    /// Sets the write strategy (builder style).
    pub fn with_mode(mut self, mode: FileMode) -> Self {
        self.mode = Some(mode);
        self
    }

    /// Sets the anchor used by [`FileMode::Insert`] (builder style).
    pub fn with_anchor(mut self, anchor: impl Into<String>) -> Self {
        self.anchor = Some(anchor.into());
        self
    }

    /// Sets the insert position used by [`FileMode::Insert`] (builder style).
    pub fn with_position(mut self, position: Position) -> Self {
        self.position = Some(position);
        self
    }

    /// Sets the block id used by [`FileMode::Block`] (builder style).
    pub fn with_marker(mut self, marker: impl Into<String>) -> Self {
        self.marker = Some(marker.into());
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn file_new_sets_path_and_content() {
        let file = File::create("src/main.rs", "fn main() {}");
        assert_eq!(file.path, "src/main.rs");
        assert_eq!(file.content, "fn main() {}");
    }

    #[test]
    fn template_deserializes_from_toml() {
        let toml_str = r#"
[info]
name = "demo"
author = "spark"
description = "a demo template"

[options]
git = true
use_liquid = false
project_root = "demo"

[[files]]
path = "README.md"
content = "Hello"
"#;
        let template: Template = toml::from_str(toml_str).unwrap();
        let info = template.info.unwrap();
        assert_eq!(info.name.as_deref(), Some("demo"));
        assert_eq!(info.author.as_deref(), Some("spark"));

        let options = template.options.unwrap();
        assert!(options.git);
        assert_eq!(options.use_liquid, Some(false));
        assert_eq!(options.project_root, "demo");

        let files = template.files.unwrap();
        assert_eq!(files.len(), 1);
        assert_eq!(files[0].path, "README.md");
        assert_eq!(files[0].content, "Hello");
    }

    #[test]
    fn template_serializes_roundtrip() {
        let template = Template {
            info: Some(Information {
                name: Some("roundtrip".into()),
                author: Some("tester".into()),
                description: Some("desc".into()),
            }),
            options: Some(Options {
                git: false,
                use_liquid: Some(true),
                json_data: None,
                project_root: "proj".into(),
            }),
            hooks: None,
            files: Some(vec![File::create("a.txt", "b")]),
        };

        let encoded = toml::to_string(&template).unwrap();
        let decoded: Template = toml::from_str(&encoded).unwrap();
        assert_eq!(decoded.info.unwrap().name.as_deref(), Some("roundtrip"));
        assert_eq!(decoded.files.unwrap()[0].content, "b");
    }

    #[test]
    fn template_builder_and_render_in_memory() {
        let template = Template::builder()
            .with_info(Information::new(
                Some("Test".into()),
                Some("Author".into()),
                Some("Desc".into()),
            ))
            .with_file(File::create(
                "{{$DIR}}/hello.txt",
                "Hello {{$USER}} from {{$DIR}}!",
            ));

        let context = Context::new()
            .with_var("DIR", "output_dir")
            .with_var("USER", "Ferris")
            .non_interactive();

        let rendered = template.render(&context).unwrap();
        assert_eq!(rendered.len(), 1);
        assert_eq!(rendered[0].path, "output_dir/hello.txt");
        assert_eq!(rendered[0].content, "Hello Ferris from output_dir!");
    }

    #[test]
    fn template_render_non_interactive_fails_on_missing_read_variable() {
        let template =
            Template::builder().with_file(File::create("greeting.txt", "Welcome, {{$NAME:read}}!"));

        let context = Context::new().non_interactive();
        let result = template.render(&context);
        assert!(matches!(result, Err(Error::MissingVariable(var)) if var == "NAME"));
    }
}
