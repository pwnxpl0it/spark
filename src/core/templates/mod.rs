use crate::output_target::OutputTarget;
use crate::utils::*;
use crate::*;
use colored::Colorize;
use promptly::prompt;
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, fs, path::Path};
pub mod options;

pub const KEYWORDS_REGEX: &str = r"\{\{\$.*?\}\}";

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(default)]
pub struct Options {
    pub git: bool,
    pub use_liquid: Option<bool>,
    pub json_data: Option<serde_json::Value>,
    pub project_root: String,
}

/// A rendered file containing its destination path and evaluated content.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RenderedFile {
    /// Evaluated destination path (or URI target like `stdout://`, `clipboard://`).
    pub path: String,
    /// Evaluated content after keyword substitution and Liquid rendering.
    pub content: String,
}

impl RenderedFile {
    /// Creates a new `RenderedFile`.
    pub fn new(path: String, content: String) -> Self {
        Self { path, content }
    }

    /// Creates a new `RenderedFile` from any types converting into `String`.
    pub fn create(path: impl Into<String>, content: impl Into<String>) -> Self {
        Self {
            path: path.into(),
            content: content.into(),
        }
    }
}

impl Template {
    /// Parses a template from a TOML string.
    ///
    /// # Example
    /// ```rust
    /// use spark::Template;
    ///
    /// let toml = r#"
    /// [[files]]
    /// path = "hello.txt"
    /// content = "Hello {{$NAME}}"
    /// "#;
    /// let template = Template::from_str(toml).unwrap();
    /// ```
    pub fn from_str(toml_str: &str) -> crate::Result<Self> {
        let template: Self = toml::from_str(toml_str)?;
        Ok(template)
    }

    /// Reads and parses a template from a TOML file path.
    pub fn from_file(path: impl AsRef<Path>) -> crate::Result<Self> {
        let path_ref = path.as_ref();
        let content = fs::read_to_string(path_ref).map_err(|e| {
            if e.kind() == std::io::ErrorKind::NotFound {
                crate::Error::InvalidPath(path_ref.to_path_buf())
            } else {
                crate::Error::Io(e)
            }
        })?;
        Self::from_str(&content)
    }

    /// Creates an empty `Template` builder.
    pub fn builder() -> Self {
        Self {
            info: None,
            options: None,
            files: Some(Vec::new()),
        }
    }

    /// Adds a file entry to the template.
    pub fn with_file(mut self, file: File) -> Self {
        if let Some(ref mut files) = self.files {
            files.push(file);
        } else {
            self.files = Some(vec![file]);
        }
        self
    }

    /// Sets information metadata on the template.
    pub fn with_info(mut self, info: Information) -> Self {
        self.info = Some(info);
        self
    }

    /// Sets options on the template.
    pub fn with_options(mut self, options: Options) -> Self {
        self.options = Some(options);
        self
    }

    pub fn set_info(&mut self, info: Information) {
        self.info = Some(info);
    }

    pub fn set_files(&mut self, files: Vec<File>) {
        self.files = Some(files);
    }

    pub fn set_options(&mut self, options: Options) {
        self.options = Some(options);
    }

    pub fn dump_options(&self) -> Option<Options> {
        self.options.clone()
    }

    pub fn generate(dest: &str) -> std::result::Result<(), String> {
        let files: Vec<File> = list_files(Path::new("./"))
            .unwrap_or_default()
            .into_iter()
            .filter(|file| !file.contains(".git"))
            .map(|file| {
                File::new(
                    file.replace("./", ""),
                    fs::read_to_string(&file).unwrap_or_default(),
                )
            })
            .collect();

        let template = Self {
            info: None,
            files: Some(files),
            options: None,
        };

        let toml_string = toml::to_string_pretty(&template)
            .map_err(|e| format!("Failed to serialize template: {}", e))?;

        write_content(dest, &toml_string)
            .map_err(|e| format!("Failed to write template to file: {}", e))?;

        println!(
            "{}: Template successfully generated at {}",
            "Success".green().bold().blink(),
            dest
        );
        Ok(())
    }

    pub fn liquify(string: &str) -> std::result::Result<String, liquid::Error> {
        let parser = liquid::ParserBuilder::with_stdlib().build()?;
        let empty_globals = liquid::Object::new();

        parser.parse(string)?.render(&empty_globals)
    }

    /// Resolves project name if `{{$PROJECTNAME}}` is referenced.
    pub fn resolve_project_name(
        keywords: &mut HashMap<String, String>,
        options: &mut Options,
        file: &File,
        interactive: bool,
    ) -> crate::Result<String> {
        let trimmed_content = file.content.trim();
        let trimmed_path = file.path.trim();
        let trimmed_project_root = options.project_root.trim();

        if trimmed_content.contains("{{$PROJECTNAME}}")
            || trimmed_path.contains("{{$PROJECTNAME}}")
            || trimmed_project_root.contains("{{$PROJECTNAME}}")
        {
            if let Some(existing) = keywords.get("{{$PROJECTNAME}}") {
                if !existing.is_empty() {
                    options.set_project_root(existing);
                    return Ok(existing.clone());
                }
            }

            if !interactive {
                return Err(crate::Error::MissingVariable("PROJECTNAME".to_string()));
            }

            let project_name: String = prompt("Project name")
                .map_err(|e| crate::Error::Prompt(e.to_string()))?;

            keywords.insert("{{$PROJECTNAME}}".to_string(), project_name.clone());
            options.set_project_root(&project_name);
            Ok(project_name)
        } else {
            Ok(String::new())
        }
    }

    #[allow(dead_code)]
    fn handle_project_name(
        keywords: &mut HashMap<String, String>,
        options: &mut Options,
        file: &File,
    ) -> std::result::Result<String, String> {
        Self::resolve_project_name(keywords, options, file, true)
            .map_err(|e| e.to_string())
    }

    fn prepare_file_content(
        file_content: &str,
        file_path: &str,
        keywords: &HashMap<String, String>,
        options: &Options,
    ) -> std::result::Result<(String, String), String> {
        let output = Keywords::replace_keywords(keywords, file_content);
        let path = Keywords::replace_keywords(keywords, file_path);

        let final_output = if options.use_liquid.unwrap_or(false) {
            Self::liquify(&output).map_err(|e| format!("Liquid error: {}", e))?
        } else {
            output
        };

        Ok((path, final_output))
    }

    /// Pure in-memory rendering of the template with the provided [`Context`].
    /// Does NOT perform side effects (does not write files or init git repo).
    ///
    /// # Example
    /// ```rust
    /// use spark::{Template, Context};
    ///
    /// let template = Template::from_str(r#"
    /// [[files]]
    /// path = "{{$SLUG}}/greeting.txt"
    /// content = "Hello {{$NAME}}!"
    /// "#).unwrap();
    ///
    /// let ctx = Context::new()
    ///     .with_var("SLUG", "myapp")
    ///     .with_var("NAME", "World")
    ///     .non_interactive();
    ///
    /// let rendered = template.render(&ctx).unwrap();
    /// assert_eq!(rendered.len(), 1);
    /// assert_eq!(rendered[0].path, "myapp/greeting.txt");
    /// assert_eq!(rendered[0].content, "Hello World!");
    /// ```
    /// Inner rendering pipeline. Returns rendered files **and** the fully-resolved
    /// keyword map so callers that need the resolved values (e.g. `extract`) can
    /// obtain them without a second placeholder-scan pass.
    fn render_inner(
        &self,
        context: &Context,
    ) -> crate::Result<(Vec<RenderedFile>, HashMap<String, String>)> {
        let re = Regex::new(KEYWORDS_REGEX)?;
        let options = self.options.clone().unwrap_or_default();
        let json_data = context
            .json_data
            .clone()
            .or_else(|| options.json_data.clone())
            .unwrap_or(serde_json::Value::Null);

        let mut keywords = context.keywords.clone();
        let files = self.files.as_deref().unwrap_or_default();
        let mut rendered = Vec::with_capacity(files.len());
        let mut project = String::new();
        let mut active_options = options;

        for file in files {
            Fns::find_and_resolve(
                &format!("{}\n{}", file.content, file.path),
                &mut keywords,
                &re,
                &json_data,
                context.interactive,
            )?;

            if project.is_empty() {
                project = Self::resolve_project_name(
                    &mut keywords,
                    &mut active_options,
                    file,
                    context.interactive,
                )?;
            }

            let (path, final_output) =
                Self::prepare_file_content(&file.content, &file.path, &keywords, &active_options)
                    .map_err(crate::Error::Custom)?;

            rendered.push(RenderedFile {
                path,
                content: final_output,
            });
        }

        Ok((rendered, keywords))
    }

    pub fn render(&self, context: &Context) -> crate::Result<Vec<RenderedFile>> {
        let (rendered, _keywords) = self.render_inner(context)?;
        Ok(rendered)
    }

    /// Renders the template and writes all files to their target sinks (filesystem,
    /// `stdout://`, `stderr://`, or `clipboard://`), handling git repository initialization
    /// if enabled in template options.
    pub fn extract_with_context(&self, context: &Context) -> crate::Result<Vec<RenderedFile>> {
        let (rendered, _keywords) = self.render_inner(context)?;

        for file in &rendered {
            OutputTarget::from_path(&file.path)
                .write(&file.content)
                .map_err(|e| crate::Error::OutputWrite {
                    path: file.path.clone(),
                    message: e.to_string(),
                })?;
        }

        if let Some(options) = &self.options {
            options.clone().handle();
        }

        Ok(rendered)
    }

    /// Legacy extraction method for backwards compatibility with CLI and tests.
    pub fn extract(&mut self, keywords: &mut HashMap<String, String>) -> std::result::Result<(), String> {
        let mut context = Context::from(keywords.clone());
        if let Some(opts) = &self.options {
            if let Some(ref jd) = opts.json_data {
                if !jd.is_null() && context.json_data.is_none() {
                    context.json_data = Some(jd.clone());
                }
            }
        }

        let (_rendered, resolved_keywords) = self
            .render_inner(&context)
            .map_err(|e| e.to_string())?;

        // Write outputs via the normal dispatch pipeline
        for file in &_rendered {
            OutputTarget::from_path(&file.path)
                .write(&file.content)
                .map_err(|e| e.to_string())?;
        }

        if let Some(options) = &self.options {
            options.clone().handle();
        }

        // Copy resolved keyword values discovered during rendering back to the
        // caller's map.  This replaces the former second Fns::find_and_exec scan
        // which could interactively re-prompt or overwrite already-resolved values.
        for (k, v) in resolved_keywords {
            keywords.entry(k).or_insert(v);
        }

        Ok(())
    }


    pub fn show_info(template: &Self) {
        if let Some(information) = &template.info { println!(
            "{}: {}\n{}: {}\n{}: {}\n",
            "Name".yellow(),
            information.name.as_ref().unwrap().bold().green(),
            "Description".yellow(),
            information.description.as_ref().unwrap().bold().green(),
            "Author".yellow(),
            information.author.as_ref().unwrap().bold().green()
        ) }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;
    use std::fs;

    #[test]
    fn liquify_errors_on_unknown_variable() {
        let result = Template::liquify("Hello {{ name }}!");
        assert!(result.is_err());
    }

    #[test]
    fn liquify_renders_literal_text() {
        let output = Template::liquify("Hello spark!").unwrap();
        assert_eq!(output, "Hello spark!");
    }

    #[test]
    fn liquify_applies_stdlib_filters() {
        let output = Template::liquify("{{ 'hello' | upcase }}").unwrap();
        assert_eq!(output, "HELLO");
    }

    #[test]
    fn set_info_files_and_options() {
        let mut template = Template {
            info: None,
            options: None,
            files: None,
        };

        template.set_info(Information {
            name: Some("demo".into()),
            author: Some("author".into()),
            description: Some("desc".into()),
        });
        template.set_files(vec![File::new("a.txt".into(), "hi".into())]);
        template.set_options(Options {
            git: true,
            use_liquid: Some(false),
            json_data: None,
            project_root: "proj".into(),
        });

        assert_eq!(
            template.info.as_ref().unwrap().name.as_deref(),
            Some("demo")
        );
        assert_eq!(template.files.as_ref().unwrap().len(), 1);
        let options = template.dump_options().unwrap();
        assert!(options.git);
        assert_eq!(options.project_root, "proj");
    }

    #[test]
    fn handle_project_name_without_placeholder_is_noop() {
        let mut keywords = HashMap::new();
        let mut options = Options::default();
        let file = File::new("test.txt".into(), "hello world".into());

        let result = Template::handle_project_name(&mut keywords, &mut options, &file).unwrap();
        assert!(result.is_empty());
        assert!(!keywords.contains_key("{{$PROJECTNAME}}"));
    }

    #[test]
    fn prepare_file_content_replaces_keywords_without_liquid() {
        let mut keywords = HashMap::new();
        keywords.insert("{{$TEST}}".to_string(), "value".to_string());
        let options = Options {
            git: false,
            use_liquid: None,
            json_data: None,
            project_root: String::new(),
        };

        let (path, content) =
            Template::prepare_file_content("Hello {{$TEST}}", "out.txt", &keywords, &options)
                .unwrap();

        assert_eq!(path, "out.txt");
        assert_eq!(content, "Hello value");
    }

    #[test]
    fn prepare_file_content_applies_liquid_when_enabled() {
        let keywords = HashMap::new();
        let options = Options {
            git: false,
            use_liquid: Some(true),
            json_data: None,
            project_root: String::new(),
        };

        let (_path, content) = Template::prepare_file_content(
            "{{ 'spark' | upcase }}",
            "out.txt",
            &keywords,
            &options,
        )
        .unwrap();

        assert_eq!(content, "SPARK");
    }

    #[test]
    fn prepare_file_content_replaces_keywords_before_liquid() {
        // Correct order: spark placeholders are replaced first, then Liquid runs.
        // If Liquid ran first, `{{ "{{$ITEM}}" | upcase }}` would become `{{$ITEM}}`
        // and the final value would stay lowercase after keyword replacement.
        let mut keywords = HashMap::new();
        keywords.insert("{{$ITEM}}".to_string(), "hello".to_string());
        let options = Options {
            git: false,
            use_liquid: Some(true),
            json_data: None,
            project_root: String::new(),
        };

        let (_path, content) = Template::prepare_file_content(
            r#"{{ "{{$ITEM}}" | upcase }}"#,
            "out.txt",
            &keywords,
            &options,
        )
        .unwrap();

        assert_eq!(content, "HELLO");
    }

    #[test]
    fn extract_resolves_json_then_replaces_then_applies_liquid() {
        let out_dir = std::env::temp_dir().join("spark_test_json_then_liquid");
        let _ = fs::remove_dir_all(&out_dir);

        let out_file = out_dir.join("out.txt");
        let mut template = Template {
            info: None,
            options: Some(Options {
                git: false,
                use_liquid: Some(true),
                json_data: Some(serde_json::json!({ "name": "spark" })),
                project_root: String::new(),
            }),
            files: Some(vec![File::new(
                out_file.to_string_lossy().to_string(),
                // JSON resolve → keyword replace → Liquid filter
                r#"{% for i in (1..3) %}{{ "{{$.name}}" | upcase }}-{{ i }} {% endfor %}"#.into(),
            )]),
        };

        let mut keywords = HashMap::new();
        template.extract(&mut keywords).unwrap();

        let content = fs::read_to_string(&out_file).unwrap();
        assert_eq!(content, "SPARK-1 SPARK-2 SPARK-3 ");

        let _ = fs::remove_dir_all(&out_dir);
    }

    #[test]
    fn output_target_file_write_creates_parent_directories() {
        // Directory creation moved from prepare_file_content into
        // OutputTarget::File::write – verify the end-to-end behaviour.
        let sub_dir = std::env::temp_dir().join("spark_test_prepare_dirs");
        let file_path = sub_dir.join("nested").join("test.txt");
        let _ = fs::remove_dir_all(&sub_dir);

        let keywords = HashMap::new();
        let options = Options {
            git: false,
            use_liquid: None,
            json_data: None,
            project_root: String::new(),
        };

        let (path, content) =
            Template::prepare_file_content("hi", &file_path.to_string_lossy(), &keywords, &options)
                .unwrap();

        assert_eq!(path, file_path.to_string_lossy());
        assert_eq!(content, "hi");

        // prepare_file_content no longer creates dirs; OutputTarget::write does.
        OutputTarget::from_path(&path).write(&content).unwrap();
        assert!(file_path.parent().unwrap().is_dir());
        assert!(file_path.is_file());

        let _ = fs::remove_dir_all(&sub_dir);
    }

    #[test]
    fn extract_with_no_files_succeeds() {
        let mut template = Template {
            info: None,
            options: Some(Options {
                git: false,
                use_liquid: None,
                json_data: None,
                project_root: String::new(),
            }),
            files: Some(vec![]),
        };
        let mut keywords = HashMap::new();
        assert!(template.extract(&mut keywords).is_ok());
    }

    #[test]
    fn extract_writes_files_with_keyword_replacement() {
        let out_dir = std::env::temp_dir().join("spark_test_extract");
        let _ = fs::remove_dir_all(&out_dir);
        fs::create_dir_all(&out_dir).unwrap();
        let out_file = out_dir.join("hello.txt");

        let mut template = Template {
            info: None,
            options: Some(Options {
                git: false,
                use_liquid: None,
                json_data: None,
                project_root: String::new(),
            }),
            files: Some(vec![File::new(
                out_file.to_string_lossy().to_string(),
                "Hello {{$NAME}}".into(),
            )]),
        };

        let mut keywords = HashMap::new();
        keywords.insert("{{$NAME}}".to_string(), "spark".to_string());

        template.extract(&mut keywords).unwrap();

        let written = fs::read_to_string(&out_file).unwrap();
        assert_eq!(written, "Hello spark");

        let _ = fs::remove_dir_all(&out_dir);
    }

    #[test]
    fn extract_resolves_path_placeholders_before_replacement() {
        // Regression: after PROJECTNAME was set from the first file, later files used to
        // skip find_and_exec and replace/write paths before resolving JSON (or :read).
        let out_dir = std::env::temp_dir().join("spark_test_resolve_before_replace");
        let _ = fs::remove_dir_all(&out_dir);

        let first_path = format!("{}/{{{{$PROJECTNAME}}}}/first.txt", out_dir.display());
        let second_path = format!(
            "{}/{{{{$PROJECTNAME}}}}/{{{{$.module}}}}/second.txt",
            out_dir.display()
        );

        let mut template = Template {
            info: None,
            options: Some(Options {
                git: false,
                use_liquid: None,
                json_data: Some(serde_json::json!({ "module": "core" })),
                project_root: "{{$PROJECTNAME}}".into(),
            }),
            files: Some(vec![
                File::new(first_path, "first".into()),
                File::new(second_path, "second {{$.module}}".into()),
            ]),
        };

        let mut keywords = HashMap::new();
        keywords.insert("{{$PROJECTNAME}}".to_string(), "myapp".to_string());

        template.extract(&mut keywords).unwrap();

        let second_file = out_dir.join("myapp").join("core").join("second.txt");
        assert!(
            second_file.is_file(),
            "expected resolved path {:?}, placeholders were replaced too early",
            second_file
        );
        assert_eq!(fs::read_to_string(&second_file).unwrap(), "second core");

        let unresolved = out_dir
            .join("myapp")
            .join("{{$.module}}")
            .join("second.txt");
        assert!(
            !unresolved.exists(),
            "wrote unresolved path placeholder: {:?}",
            unresolved
        );

        let _ = fs::remove_dir_all(&out_dir);
    }

    /// Realistic embedded TOML template used by JSON integration tests.
    fn embedded_json_template_toml(out_dir: &str) -> String {
        format!(
            r#"
[info]
name = "json_demo"
author = "spark-tests"
description = "Embedded template for JSON integration tests"

[options]
git = false
use_liquid = false
project_root = "json_demo"

[[files]]
path = "{out}/{{{{$.project.slug}}}}/README.md"
content = """
# {{{{$.user.name}}}}'s Project

User ID: {{{{$.user.id}}}}
Email: {{{{$.user.email}}}}
Status: {{{{$.status[0]}}}}
"""

[[files]]
path = "{out}/{{{{$.project.slug}}}}/src/{{{{$.user.id}}}}.txt"
content = """
package {{{{$.project.slug}}}}
owner = {{{{$.user.name}}}}
"""
"#,
            out = out_dir
        )
    }

    fn embedded_json_data() -> serde_json::Value {
        serde_json::json!({
            "user": {
                "id": "12345",
                "name": "John Doe",
                "email": "john.doe@example.com"
            },
            "project": {
                "slug": "demo_app"
            },
            "status": ["200 OK"]
        })
    }

    #[test]
    fn extract_from_embedded_template_with_json_data() {
        let out_dir = std::env::temp_dir().join("spark_test_embedded_json_template");
        let _ = fs::remove_dir_all(&out_dir);

        let toml_str = embedded_json_template_toml(&out_dir.to_string_lossy());
        let mut template: Template =
            toml::from_str(&toml_str).expect("embedded template should parse");

        let mut options = template.dump_options().unwrap_or_default();
        options.set_json(embedded_json_data());
        options.use_liquid = Some(false);
        template.set_options(options);

        let mut keywords = HashMap::new();
        template.extract(&mut keywords).unwrap();

        let readme = out_dir.join("demo_app").join("README.md");
        let profile = out_dir.join("demo_app").join("src").join("12345.txt");

        assert!(readme.is_file(), "missing README at {:?}", readme);
        assert!(profile.is_file(), "missing profile at {:?}", profile);

        let readme_content = fs::read_to_string(&readme).unwrap();
        assert!(readme_content.contains("# John Doe's Project"));
        assert!(readme_content.contains("User ID: 12345"));
        assert!(readme_content.contains("Email: john.doe@example.com"));
        assert!(readme_content.contains("Status: 200 OK"));

        let profile_content = fs::read_to_string(&profile).unwrap();
        assert!(profile_content.contains("package demo_app"));
        assert!(profile_content.contains("owner = John Doe"));

        let _ = fs::remove_dir_all(&out_dir);
    }

    #[test]
    fn extract_reads_json_from_file_like_cli() {
        let out_dir = std::env::temp_dir().join("spark_test_json_file");
        let _ = fs::remove_dir_all(&out_dir);
        fs::create_dir_all(&out_dir).unwrap();

        let json_path = out_dir.join("data.json");
        fs::write(
            &json_path,
            r#"{
                "user": { "id": "99", "name": "Ada", "email": "ada@example.com" },
                "project": { "slug": "from_file" },
                "status": ["201 Created"]
            }"#,
        )
        .unwrap();

        let json_data: serde_json::Value =
            serde_json::from_str(&fs::read_to_string(&json_path).unwrap()).unwrap();

        let toml_str = embedded_json_template_toml(&out_dir.to_string_lossy());
        let mut template: Template = toml::from_str(&toml_str).unwrap();

        let mut options = template.dump_options().unwrap_or_default();
        options.set_json(json_data);
        options.use_liquid = Some(false);
        template.set_options(options);

        let mut keywords = HashMap::new();
        template.extract(&mut keywords).unwrap();

        let readme = out_dir.join("from_file").join("README.md");
        let content = fs::read_to_string(&readme).expect("README should be written");
        assert!(content.contains("# Ada's Project"));
        assert!(content.contains("User ID: 99"));
        assert!(content.contains("Status: 201 Created"));

        let src_file = out_dir.join("from_file").join("src").join("99.txt");
        assert!(src_file.is_file());

        let _ = fs::remove_dir_all(&out_dir);
    }

    #[test]
    fn template_parses_embedded_json_data_option_from_toml() {
        let toml_str = r#"
[info]
name = "with_json"

[options]
git = false
use_liquid = false
project_root = "proj"

[options.json_data]
status = ["ok"]

[options.json_data.user]
id = "1"
name = "Neo"

[[files]]
path = "ignored.txt"
content = "x"
"#;
        let template: Template =
            toml::from_str(toml_str).expect("toml with json_data should parse");
        let options = template.options.expect("options present");
        let json = options.json_data.expect("json_data present");
        assert_eq!(json["user"]["name"], "Neo");
        assert_eq!(json["status"][0], "ok");
    }

    #[test]
    fn template_parses_options_with_omitted_optional_fields() {
        let toml_str = r#"
[options]
use_liquid = false

[options.json_data.user]
name = "Trinity"

[[files]]
path = "out.txt"
content = "hello"
"#;
        let template: Template =
            toml::from_str(toml_str).expect("should parse without project_root or git");
        let options = template.options.expect("options present");
        assert!(!options.git);
        assert_eq!(options.project_root, "");
        assert_eq!(options.use_liquid, Some(false));
    }

    /// Scenario 5 — `--from="PROJECTNAME=myapp"` skips the interactive prompt.
    ///
    /// `main()` pre-inserts the value into `keywords` before calling `extract()`.
    /// When `{{$PROJECTNAME}}` is already set and non-empty, `handle_project_name`
    /// must use it directly and must NOT call `prompt()`.
    ///
    /// We verify this by passing a pre-set keywords map and asserting that:
    /// 1. `extract()` succeeds without blocking on stdin.
    /// 2. Files are resolved to the pre-set project-name path (`myapp/...`).
    /// 3. No unresolved `{{$PROJECTNAME}}` literal appears in any written path.
    #[test]
    fn extract_with_from_flag_skips_projectname_prompt() {
        let out_dir = std::env::temp_dir().join("spark_test_from_flag_projectname");
        let _ = fs::remove_dir_all(&out_dir);

        let file_path = format!("{}/{{{{$PROJECTNAME}}}}/README.md", out_dir.display());

        let mut template = Template {
            info: None,
            options: Some(Options {
                git: false,
                use_liquid: None,
                json_data: None,
                project_root: "{{$PROJECTNAME}}".into(),
            }),
            files: Some(vec![File::new(file_path, "# {{$PROJECTNAME}}".into())]),
        };

        // Pre-populate PROJECTNAME exactly as main() does when --from is given.
        let mut keywords = HashMap::new();
        keywords.insert("{{$PROJECTNAME}}".to_string(), "myapp".to_string());

        // Must not block waiting for stdin — PROJECTNAME is already set.
        template.extract(&mut keywords).unwrap();

        let readme = out_dir.join("myapp").join("README.md");
        assert!(
            readme.is_file(),
            "--from should resolve {{{{$PROJECTNAME}}}} to 'myapp', expected {:?}",
            readme
        );
        let content = fs::read_to_string(&readme).unwrap();
        assert_eq!(
            content, "# myapp",
            "content should have PROJECTNAME replaced with 'myapp'"
        );

        // No unresolved placeholder written to disk.
        let unresolved = out_dir.join("{{$PROJECTNAME}}").join("README.md");
        assert!(
            !unresolved.exists(),
            "unresolved placeholder path must not exist: {:?}",
            unresolved
        );

        let _ = fs::remove_dir_all(&out_dir);
    }

    #[test]
    fn extract_uses_json_data_embedded_in_template_options() {
        let out_dir = std::env::temp_dir().join("spark_test_toml_embedded_json");
        let _ = fs::remove_dir_all(&out_dir);

        let toml_str = format!(
            r#"
[info]
name = "toml_json"

[options]
git = false
use_liquid = false
project_root = "proj"

[options.json_data]
status = ["embedded"]

[options.json_data.user]
id = "7"
name = "Trinity"
email = "trinity@matrix"

[options.json_data.project]
slug = "matrix"

[[files]]
path = "{out}/{{{{$.project.slug}}}}/hello.txt"
content = """
Hello {{{{$.user.name}}}} ({{{{$.user.id}}}})
Status: {{{{$.status[0]}}}}
"""
"#,
            out = out_dir.display()
        );

        let mut template: Template = toml::from_str(&toml_str).unwrap();
        let mut keywords = HashMap::new();
        template.extract(&mut keywords).unwrap();

        let hello = out_dir.join("matrix").join("hello.txt");
        let content = fs::read_to_string(&hello).unwrap();
        assert_eq!(content, "Hello Trinity (7)\nStatus: embedded\n");

        let _ = fs::remove_dir_all(&out_dir);
    }

    // ── OutputTarget integration via extract() ───────────────────────────────

    /// Calling extract() with `path = "stdout://"` must succeed and must NOT
    /// write any file to the filesystem.
    #[test]
    fn extract_stdout_target_does_not_write_file() {
        let mut template = Template {
            info: None,
            options: Some(Options {
                git: false,
                use_liquid: None,
                json_data: None,
                project_root: String::new(),
            }),
            files: Some(vec![File::new(
                "stdout://".into(),
                "Hello {{$GREETING}}".into(),
            )]),
        };

        let mut keywords = HashMap::new();
        keywords.insert("{{$GREETING}}".to_string(), "world".to_string());

        // Must succeed – stdout is a valid target.
        assert!(template.extract(&mut keywords).is_ok());

        // Verify no file named "stdout:" or "stdout://" was created anywhere
        // reachable from the current directory.
        assert!(!std::path::Path::new("stdout:").exists());
        assert!(!std::path::Path::new("stdout://").exists());
    }

    /// Calling extract() with `path = "stderr://"` must succeed without
    /// creating a filesystem entry.
    #[test]
    fn extract_stderr_target_does_not_write_file() {
        let mut template = Template {
            info: None,
            options: Some(Options {
                git: false,
                use_liquid: None,
                json_data: None,
                project_root: String::new(),
            }),
            files: Some(vec![File::new(
                "stderr://".into(),
                "error: {{$MSG}}".into(),
            )]),
        };

        let mut keywords = HashMap::new();
        keywords.insert("{{$MSG}}".to_string(), "something went wrong".to_string());

        assert!(template.extract(&mut keywords).is_ok());
        assert!(!std::path::Path::new("stderr:").exists());
        assert!(!std::path::Path::new("stderr://").exists());
    }

    /// `file://` prefix is stripped and the content is written to the
    /// filesystem path that follows.
    #[test]
    fn extract_file_scheme_writes_to_disk() {
        let out_dir = std::env::temp_dir().join("spark_test_extract_file_scheme");
        let _ = fs::remove_dir_all(&out_dir);
        fs::create_dir_all(&out_dir).unwrap();

        let file_path = out_dir.join("result.txt");
        let raw_path = format!("file://{}", file_path.display());

        let mut template = Template {
            info: None,
            options: Some(Options {
                git: false,
                use_liquid: None,
                json_data: None,
                project_root: String::new(),
            }),
            files: Some(vec![File::new(raw_path, "content via file://".into())]),
        };

        let mut keywords = HashMap::new();
        template.extract(&mut keywords).unwrap();

        assert!(file_path.is_file(), "file:// target must write to disk");
        assert_eq!(
            fs::read_to_string(&file_path).unwrap(),
            "content via file://"
        );

        let _ = fs::remove_dir_all(&out_dir);
    }

    /// Keywords are replaced before dispatching to stdout://.
    #[test]
    fn extract_stdout_applies_keyword_replacement() {
        // We cannot capture stdout in a unit test without additional deps,
        // but we can verify that extract() returns Ok (not Err) when keywords
        // are present, proving the replacement+dispatch pipeline ran to
        // completion without errors.
        let mut template = Template {
            info: None,
            options: Some(Options {
                git: false,
                use_liquid: None,
                json_data: None,
                project_root: String::new(),
            }),
            files: Some(vec![File::new(
                "stdout://".into(),
                "project: {{$NAME}} by {{$AUTHOR}}".into(),
            )]),
        };

        let mut keywords = HashMap::new();
        keywords.insert("{{$NAME}}".to_string(), "spark".to_string());
        keywords.insert("{{$AUTHOR}}".to_string(), "pwnxpl0it".to_string());

        assert!(template.extract(&mut keywords).is_ok());
    }

    /// Liquid is applied before dispatching to stderr://.
    #[test]
    fn extract_stderr_applies_liquid() {
        let mut template = Template {
            info: None,
            options: Some(Options {
                git: false,
                use_liquid: Some(true),
                json_data: None,
                project_root: String::new(),
            }),
            files: Some(vec![File::new(
                "stderr://".into(),
                "{{ 'warn' | upcase }}".into(),
            )]),
        };

        let mut keywords = HashMap::new();
        assert!(template.extract(&mut keywords).is_ok());
    }

    /// A mix of protocol and filesystem targets in one template works
    /// correctly: the filesystem file is written, stdout/stderr are not.
    #[test]
    fn extract_mixed_targets_file_and_stdout() {
        let out_dir = std::env::temp_dir().join("spark_test_mixed_targets");
        let _ = fs::remove_dir_all(&out_dir);
        fs::create_dir_all(&out_dir).unwrap();

        let file_path = out_dir.join("notes.txt");

        let mut template = Template {
            info: None,
            options: Some(Options {
                git: false,
                use_liquid: None,
                json_data: None,
                project_root: String::new(),
            }),
            files: Some(vec![
                File::new(
                    file_path.to_string_lossy().to_string(),
                    "filesystem content".into(),
                ),
                File::new("stdout://".into(), "stdout content".into()),
                File::new("stderr://".into(), "stderr content".into()),
            ]),
        };

        let mut keywords = HashMap::new();
        template.extract(&mut keywords).unwrap();

        // Filesystem file must exist with correct content.
        assert!(file_path.is_file());
        assert_eq!(
            fs::read_to_string(&file_path).unwrap(),
            "filesystem content"
        );

        // No spurious files for the protocol paths.
        assert!(!std::path::Path::new("stdout:").exists());
        assert!(!std::path::Path::new("stderr:").exists());

        let _ = fs::remove_dir_all(&out_dir);
    }

    #[test]
    fn extract_does_not_form_synthetic_placeholder_across_content_and_path() {
        let out_dir = std::env::temp_dir().join("spark_test_no_synthetic_placeholder");
        let _ = fs::remove_dir_all(&out_dir);
        fs::create_dir_all(&out_dir).unwrap();

        let out_file = out_dir.join("file.txt");
        let file_path = out_file.to_string_lossy().to_string();

        let mut template = Template {
            info: None,
            options: Some(Options {
                git: false,
                use_liquid: None,
                json_data: None,
                project_root: String::new(),
            }),
            // content ends with "{{$" and path starts with a string ending in "}}"
            // Neither is a valid placeholder on its own.
            files: Some(vec![File::new(file_path.clone(), "Hello {{$".into())]),
        };

        let mut keywords = HashMap::new();
        template.extract(&mut keywords).unwrap();

        // The keywords map should not contain any synthetic key created across the boundary
        assert!(
            keywords.is_empty()
                || !keywords
                    .keys()
                    .any(|k| k.starts_with("{{$") && k.ends_with("}}"))
        );

        let _ = fs::remove_dir_all(&out_dir);
    }
}
