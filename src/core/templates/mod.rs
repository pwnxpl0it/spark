use crate::output_target::OutputTarget;
use crate::utils::*;
use crate::*;
use colored::Colorize;
use indexmap::IndexMap;
use promptly::prompt;
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

impl std::str::FromStr for Template {
    type Err = crate::Error;

    /// Parses a template from a TOML string, so `toml_str.parse::<Template>()` works.
    fn from_str(toml_str: &str) -> crate::Result<Self> {
        let template: Self = toml::from_str(toml_str)?;
        Ok(template)
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
    #[allow(clippy::should_implement_trait)]
    pub fn from_str(toml_str: &str) -> crate::Result<Self> {
        <Self as std::str::FromStr>::from_str(toml_str)
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

    /// Generates a template at `dest` from every file under the current directory.
    pub fn generate(dest: &str) -> std::result::Result<(), String> {
        Self::generate_from(Path::new("./"), dest)
    }

    /// Generates a template at `dest` from every file under `source`.
    /// Paths in the template are relative to `source`.
    pub fn generate_from(source: &Path, dest: &str) -> std::result::Result<(), String> {
        let dest_abs = fs::canonicalize(dest).ok();
        let prefix = source.to_string_lossy().to_string();
        let files: Vec<File> = list_files(source)
            .unwrap_or_default()
            .into_iter()
            // Never capture the template file we are about to write.
            .filter(|file| dest_abs.is_none() || fs::canonicalize(file).ok() != dest_abs)
            .filter_map(|file| {
                let content = match fs::read_to_string(&file) {
                    Ok(c) => c,
                    Err(_) => return None,
                };

                if content.contains('\0') {
                    return None;
                }

                // Escape any placeholder-looking text so the generated
                // template reproduces the captured files verbatim.
                // Template paths always use '/' so a template generated on
                // Windows renders on any platform.
                let relative = file
                    .strip_prefix(&prefix)
                    .unwrap_or(&file)
                    .trim_start_matches(['/', '\\'])
                    .replace('\\', "/");
                Some(File::new(
                    Keywords::escape(&relative),
                    Keywords::escape(&content),
                ))
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

        eprintln!(
            "{}: Template successfully generated at {}",
            "Success".green().bold(),
            dest
        );
        Ok(())
    }

    /// Renders `string` as a Liquid template with no globals in scope.
    pub fn liquify(string: &str) -> std::result::Result<String, liquid::Error> {
        Self::liquify_with(string, &liquid::Object::new())
    }

    /// Renders `string` as a Liquid template with `globals` available as
    /// top-level variables (e.g. the template's own `info`/`options` metadata).
    pub fn liquify_with(
        string: &str,
        globals: &liquid::Object,
    ) -> std::result::Result<String, liquid::Error> {
        let parser = liquid::ParserBuilder::with_stdlib().build()?;
        parser.parse(string)?.render(globals)
    }

    /// Builds the keyword/value pairs exposing the template's own `[info]` and
    /// `[options]` metadata as `{{$INFO_*}}` / `{{$OPTIONS_*}}` placeholders.
    /// Used both to seed the keyword map and (indirectly) to document the
    /// available tokens.
    fn metadata_keywords(info: &Option<Information>, options: &Options) -> Vec<(String, String)> {
        let mut pairs = Vec::new();
        if let Some(info) = info {
            if let Some(name) = &info.name {
                pairs.push((Keywords::from("INFO_NAME", None), name.clone()));
            }
            if let Some(author) = &info.author {
                pairs.push((Keywords::from("INFO_AUTHOR", None), author.clone()));
            }
            if let Some(description) = &info.description {
                pairs.push((
                    Keywords::from("INFO_DESCRIPTION", None),
                    description.clone(),
                ));
            }
        }
        pairs.push((Keywords::from("OPTIONS_GIT", None), options.git.to_string()));
        pairs.push((
            Keywords::from("OPTIONS_USE_LIQUID", None),
            options.use_liquid.unwrap_or(false).to_string(),
        ));
        pairs
    }

    /// Builds the Liquid globals object exposing the template's own `[info]` and
    /// `[options]` metadata as the `info` and `options` variables, so templates
    /// can branch on them (e.g. `{% if options.git %}`).
    fn metadata_globals(info: &Option<Information>, options: &Options) -> liquid::Object {
        use liquid::model::Value;

        let info = info.clone().unwrap_or_default();

        let mut info_obj = liquid::Object::new();
        info_obj.insert("name".into(), Value::scalar(info.name.unwrap_or_default()));
        info_obj.insert(
            "author".into(),
            Value::scalar(info.author.unwrap_or_default()),
        );
        info_obj.insert(
            "description".into(),
            Value::scalar(info.description.unwrap_or_default()),
        );

        let mut options_obj = liquid::Object::new();
        options_obj.insert("git".into(), Value::scalar(options.git));
        options_obj.insert(
            "use_liquid".into(),
            Value::scalar(options.use_liquid.unwrap_or(false)),
        );

        let mut globals = liquid::Object::new();
        globals.insert("info".into(), Value::Object(info_obj));
        globals.insert("options".into(), Value::Object(options_obj));
        globals
    }

    /// Resolves the project name if `{{$PROJECTNAME}}` is referenced by the file,
    /// its path, or `options.project_root`, prompting when interactive.
    ///
    /// Only the keyword map is updated; `project_root` keeps its template form and
    /// is substituted once every keyword is known (see `render_inner`), so values
    /// such as `"{{$PROJECTNAME}}-api"` or `"/abs/{{$PROJECTNAME}}"` work as written.
    pub fn resolve_project_name(
        keywords: &mut HashMap<String, String>,
        options: &Options,
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
                    return Ok(existing.clone());
                }
            }

            if !interactive {
                return Err(crate::Error::MissingVariable("PROJECTNAME".to_string()));
            }

            let project_name: String =
                prompt("Project name").map_err(|e| crate::Error::Prompt(e.to_string()))?;

            keywords.insert("{{$PROJECTNAME}}".to_string(), project_name.clone());
            Ok(project_name)
        } else {
            Ok(String::new())
        }
    }

    fn prepare_file_content(
        file_content: &str,
        file_path: &str,
        keywords: &HashMap<String, String>,
        options: &Options,
        globals: &liquid::Object,
    ) -> std::result::Result<(String, String), String> {
        let output = Keywords::replace_keywords(keywords, file_content);
        let path = Keywords::replace_keywords(keywords, file_path);

        let final_output = if options.use_liquid.unwrap_or(false) {
            Self::liquify_with(&output, globals).map_err(|e| format!("Liquid error: {}", e))?
        } else {
            output
        };

        Ok((Keywords::unescape(&path), Keywords::unescape(&final_output)))
    }

    /// Inner rendering pipeline. Returns rendered files **and** the fully-resolved
    /// keyword map so callers that need the resolved values (e.g. `extract`) can
    /// obtain them without a second placeholder-scan pass.
    fn render_inner(
        &self,
        context: &Context,
    ) -> crate::Result<(Vec<RenderedFile>, HashMap<String, String>, Options)> {
        let re = Keywords::regex();
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

        // Expose the template's own `[info]`/`[options]` metadata: as
        // `{{$INFO_*}}` / `{{$OPTIONS_*}}` keywords (for every file, path, and
        // anchor/marker) and as Liquid `info`/`options` variables. Existing
        // keys (env, `[Keywords]`, `--from`) are left untouched.
        for (key, value) in Self::metadata_keywords(&self.info, &active_options) {
            keywords.entry(key).or_insert(value);
        }
        let globals = Self::metadata_globals(&self.info, &active_options);

        for file in files {
            // Scan content, path, and (for patch modes) anchor/marker so a
            // placeholder that appears *only* in anchor/marker is still resolved
            // or prompted. '\n' between parts keeps a trailing `{{$` in one part
            // from fusing with a leading `}}` in the next into a false token.
            let mut scan = format!("{}\n{}", file.content, file.path);
            if let Some(anchor) = &file.anchor {
                scan.push('\n');
                scan.push_str(anchor);
            }
            if let Some(marker) = &file.marker {
                scan.push('\n');
                scan.push_str(marker);
            }
            Fns::find_and_resolve(
                &scan,
                &mut keywords,
                re,
                &json_data,
                context.interactive,
                context.on_missing,
            )?;

            if project.is_empty() {
                project = Self::resolve_project_name(
                    &mut keywords,
                    &active_options,
                    file,
                    context.interactive,
                )?;
            }

            let (path, final_output) = Self::prepare_file_content(
                &file.content,
                &file.path,
                &keywords,
                &active_options,
                &globals,
            )
            .map_err(crate::Error::Custom)?;

            rendered.push(RenderedFile {
                path,
                content: final_output,
            });
        }

        // `project_root` may reference `{{$PROJECTNAME}}` or any other keyword;
        // resolve it with the final keyword map so the git hook sees a real path.
        active_options.project_root =
            Keywords::replace_keywords(&keywords, &active_options.project_root);

        Ok((rendered, keywords, active_options))
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
    pub fn render(&self, context: &Context) -> crate::Result<Vec<RenderedFile>> {
        let (rendered, _keywords, _options) = self.render_inner(context)?;
        Ok(rendered)
    }

    /// Renders, writes every file to its sink, and runs the git hook.
    /// Returns rendered files plus the fully-resolved keyword map.
    fn extract_inner(
        &self,
        context: &Context,
    ) -> crate::Result<(Vec<RenderedFile>, HashMap<String, String>)> {
        let (rendered, keywords, options) = self.render_inner(context)?;
        // `rendered` is produced from `self.files` in order, so index-for-index
        // the two line up; zip them to recover each entry's write mode.
        let sources = self.files.as_deref().unwrap_or_default();

        if !context.allow_outside {
            if let Some(unsafe_file) = rendered
                .iter()
                .find(|f| OutputTarget::from_path(&f.path).escapes_cwd())
            {
                return Err(crate::Error::UnsafePath(unsafe_file.path.clone()));
            }
        }

        if !context.force {
            // Only overwrite-mode entries are guarded against clobbering an
            // existing file; patch modes exist precisely to edit files that are
            // already on disk.
            let existing: Vec<String> = rendered
                .iter()
                .zip(sources.iter())
                .filter(|(_, src)| src.mode() == FileMode::Overwrite)
                .filter(|(f, _)| OutputTarget::from_path(&f.path).exists())
                .map(|(f, _)| f.path.clone())
                .collect();

            if !existing.is_empty() {
                if !context.interactive {
                    return Err(crate::Error::FileExists(existing));
                }
                eprintln!(
                    "\n{}: the following file(s) already exist:\n  {}",
                    "warning".yellow().bold(),
                    existing.join("\n  ")
                );
                let overwrite: bool = promptly::prompt_default("Overwrite them?", false)
                    .map_err(|e| crate::Error::Prompt(e.to_string()))?;
                if !overwrite {
                    return Err(crate::Error::FileExists(existing));
                }
            }
        }

        // Phase 1 — plan every write without touching disk, so a failure in any
        // patch merge aborts before a single file is written (the same
        // "validate everything first" guarantee overwrite mode already had).
        // File targets are keyed by expanded path so that several entries
        // patching the same file build on each other in memory and are written
        // once; stdout/stderr/clipboard writes are planned in order.
        let mut file_plan: IndexMap<String, (OutputTarget, String, String)> = IndexMap::new();
        let mut sink_plan: Vec<(OutputTarget, String, String)> = Vec::new();

        for (file, src) in rendered.iter().zip(sources.iter()) {
            let target = OutputTarget::from_path(&file.path);
            let mode = src.mode();

            match target.expanded_path() {
                Some(path) => {
                    let key = path.to_string_lossy().into_owned();
                    let content = if !mode.reads_existing() {
                        file.content.clone()
                    } else {
                        // Base content: a merge already planned for this path (so
                        // repeated patches on one file accumulate), else what is on
                        // disk. A missing file is valid "no content"; any other read
                        // error surfaces rather than silently discarding the file.
                        let existing = if let Some((_, planned, _)) = file_plan.get(&key) {
                            Some(planned.clone())
                        } else {
                            match fs::read_to_string(&path) {
                                Ok(contents) => Some(contents),
                                Err(e) if e.kind() == std::io::ErrorKind::NotFound => None,
                                Err(e) => {
                                    return Err(crate::Error::PatchApply {
                                        path: file.path.clone(),
                                        message: format!(
                                            "cannot read existing file to patch it: {e}"
                                        ),
                                    });
                                }
                            }
                        };
                        // Anchor/marker may carry placeholders; resolve with the
                        // final keyword map.
                        let anchor = src
                            .anchor
                            .as_deref()
                            .map(|a| Keywords::unescape(&Keywords::replace_keywords(&keywords, a)));
                        let marker = src
                            .marker
                            .as_deref()
                            .map(|m| Keywords::unescape(&Keywords::replace_keywords(&keywords, m)));
                        mode.apply(
                            existing.as_deref(),
                            &file.content,
                            anchor.as_deref(),
                            src.position(),
                            marker.as_deref(),
                        )
                        .map_err(|message| crate::Error::PatchApply {
                            path: file.path.clone(),
                            message,
                        })?
                    };
                    file_plan.insert(key, (target, content, file.path.clone()));
                }
                None => {
                    // Non-file targets (stdout://, clipboard://, …) have nothing to
                    // merge into; write the rendered fragment as-is.
                    sink_plan.push((target, file.content.clone(), file.path.clone()));
                }
            }
        }

        // Phase 2 — commit the planned writes now that every merge succeeded.
        for (target, content, display) in file_plan.values().chain(sink_plan.iter()) {
            target
                .write(content)
                .map_err(|e| crate::Error::OutputWrite {
                    path: display.clone(),
                    message: e.to_string(),
                })?;
        }

        if self.options.is_some() {
            options.handle();
        }

        Ok((rendered, keywords))
    }

    /// Renders the template and writes all files to their target sinks (filesystem,
    /// `stdout://`, `stderr://`, or `clipboard://`), handling git repository initialization
    /// if enabled in template options.
    pub fn extract_with_context(&self, context: &Context) -> crate::Result<Vec<RenderedFile>> {
        let (rendered, _keywords) = self.extract_inner(context)?;
        Ok(rendered)
    }

    /// Legacy extraction method for backwards compatibility with CLI and tests.
    ///
    /// Builds an interactive [`Context`] from `keywords`, runs
    /// [`Template::extract_with_context`], and copies every keyword resolved
    /// during rendering (prompts, JSON lookups) back into the caller's map.
    pub fn extract(
        &mut self,
        keywords: &mut HashMap<String, String>,
    ) -> std::result::Result<(), String> {
        let context = Context::from(keywords.clone());
        let (_rendered, resolved_keywords) =
            self.extract_inner(&context).map_err(|e| e.to_string())?;

        for (k, v) in resolved_keywords {
            keywords.entry(k).or_insert(v);
        }

        Ok(())
    }

    /// Returns the `[info]` section formatted for display, one `Label: value`
    /// line per field that is present. Returns `None` when there is no info
    /// or every field is missing.
    pub fn info_summary(&self) -> Option<String> {
        let information = self.info.as_ref()?;
        let fields = [
            ("Name", &information.name),
            ("Description", &information.description),
            ("Author", &information.author),
        ];
        let lines: Vec<String> = fields
            .iter()
            .filter_map(|(label, value)| {
                value
                    .as_ref()
                    .map(|v| format!("{}: {}", label.yellow(), v.bold().green()))
            })
            .collect();
        if lines.is_empty() {
            None
        } else {
            Some(lines.join("\n"))
        }
    }

    pub fn show_info(template: &Self) {
        if let Some(summary) = template.info_summary() {
            eprintln!("{}\n", summary);
        }
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
    fn resolve_project_name_without_placeholder_is_noop() {
        let mut keywords = HashMap::new();
        let options = Options::default();
        let file = File::new("test.txt".into(), "hello world".into());

        let result = Template::resolve_project_name(&mut keywords, &options, &file, true).unwrap();
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

        let (path, content) = Template::prepare_file_content(
            "Hello {{$TEST}}",
            "out.txt",
            &keywords,
            &options,
            &liquid::Object::new(),
        )
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
            &liquid::Object::new(),
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
            &liquid::Object::new(),
        )
        .unwrap();

        assert_eq!(content, "HELLO");
    }

    #[test]
    fn extract_resolves_json_then_replaces_then_applies_liquid() {
        let out_dir_tmp = tempfile::tempdir().unwrap();
        let out_dir = out_dir_tmp.path().to_path_buf();

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
    }

    #[test]
    fn output_target_file_write_creates_parent_directories() {
        // Directory creation moved from prepare_file_content into
        // OutputTarget::File::write – verify the end-to-end behaviour.
        let sub_dir_tmp = tempfile::tempdir().unwrap();
        let sub_dir = sub_dir_tmp.path().to_path_buf();
        let file_path = sub_dir.join("nested").join("test.txt");

        let keywords = HashMap::new();
        let options = Options {
            git: false,
            use_liquid: None,
            json_data: None,
            project_root: String::new(),
        };

        let (path, content) = Template::prepare_file_content(
            "hi",
            &file_path.to_string_lossy(),
            &keywords,
            &options,
            &liquid::Object::new(),
        )
        .unwrap();

        assert_eq!(path, file_path.to_string_lossy());
        assert_eq!(content, "hi");

        // prepare_file_content no longer creates dirs; OutputTarget::write does.
        OutputTarget::from_path(&path).write(&content).unwrap();
        assert!(file_path.parent().unwrap().is_dir());
        assert!(file_path.is_file());
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
        let out_dir_tmp = tempfile::tempdir().unwrap();
        let out_dir = out_dir_tmp.path().to_path_buf();
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
    }

    #[test]
    fn extract_resolves_path_placeholders_before_replacement() {
        // Regression: after PROJECTNAME was set from the first file, later files used to
        // skip find_and_exec and replace/write paths before resolving JSON (or :read).
        let out_dir_tmp = tempfile::tempdir().unwrap();
        let out_dir = out_dir_tmp.path().to_path_buf();

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
    }

    /// Realistic embedded TOML template used by JSON integration tests.
    fn embedded_json_template_toml(out_dir: &str) -> String {
        // The path is interpolated into a TOML basic string, where Windows
        // backslashes would be read as escape sequences (`\U` in `C:\Users`).
        let out_dir = out_dir.replace('\\', "/");
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
        let out_dir_tmp = tempfile::tempdir().unwrap();
        let out_dir = out_dir_tmp.path().to_path_buf();

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
    }

    #[test]
    fn extract_reads_json_from_file_like_cli() {
        let out_dir_tmp = tempfile::tempdir().unwrap();
        let out_dir = out_dir_tmp.path().to_path_buf();
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
        let out_dir_tmp = tempfile::tempdir().unwrap();
        let out_dir = out_dir_tmp.path().to_path_buf();

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
    }

    #[test]
    fn extract_uses_json_data_embedded_in_template_options() {
        let out_dir_tmp = tempfile::tempdir().unwrap();
        let out_dir = out_dir_tmp.path().to_path_buf();

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
            out = out_dir.display().to_string().replace('\\', "/")
        );

        let mut template: Template = toml::from_str(&toml_str).unwrap();
        let mut keywords = HashMap::new();
        template.extract(&mut keywords).unwrap();

        let hello = out_dir.join("matrix").join("hello.txt");
        let content = fs::read_to_string(&hello).unwrap();
        assert_eq!(content, "Hello Trinity (7)\nStatus: embedded\n");
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
        let out_dir_tmp = tempfile::tempdir().unwrap();
        let out_dir = out_dir_tmp.path().to_path_buf();
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
        let out_dir_tmp = tempfile::tempdir().unwrap();
        let out_dir = out_dir_tmp.path().to_path_buf();
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
    }

    #[test]
    fn extract_does_not_form_synthetic_placeholder_across_content_and_path() {
        let out_dir_tmp = tempfile::tempdir().unwrap();
        let out_dir = out_dir_tmp.path().to_path_buf();
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

        // The keywords map should not contain any synthetic key created across
        // the boundary. The template's own `[info]`/`[options]` metadata tokens
        // are seeded legitimately, so ignore those.
        let synthetic: Vec<&String> = keywords
            .keys()
            .filter(|k| k.starts_with("{{$") && k.ends_with("}}"))
            .filter(|k| {
                !matches!(
                    k.as_str(),
                    "{{$INFO_NAME}}"
                        | "{{$INFO_AUTHOR}}"
                        | "{{$INFO_DESCRIPTION}}"
                        | "{{$OPTIONS_GIT}}"
                        | "{{$OPTIONS_USE_LIQUID}}"
                )
            })
            .collect();
        assert!(
            synthetic.is_empty(),
            "unexpected synthetic placeholder keys: {synthetic:?}"
        );
    }

    #[test]
    fn generate_filters_binary_file_with_nul_after_1024_bytes() {
        let src_tmp = tempfile::tempdir().unwrap();
        let src = src_tmp.path();
        let temp_bin_file = src.join("spark_test_nul_after_1024.bin");
        let temp_invalid_utf8 = src.join("spark_test_invalid_utf8.dat");
        let temp_txt_file = src.join("spark_test_valid_text.txt");
        let dest_tmp = tempfile::tempdir().unwrap();
        let dest_file = dest_tmp.path().join("spark_test_generated_template.toml");

        // Create a binary file with NUL byte past 1024 bytes
        let mut bin_data = vec![b'a'; 1500];
        bin_data[1200] = 0;
        fs::write(&temp_bin_file, &bin_data).unwrap();

        // Create an invalid UTF-8 binary file
        fs::write(&temp_invalid_utf8, [0xFF, 0xFE, 0xFD]).unwrap();

        // Create a valid text file
        fs::write(&temp_txt_file, "valid text content").unwrap();

        let res = Template::generate_from(src, dest_file.to_str().unwrap());
        assert!(res.is_ok());

        let generated_content = fs::read_to_string(&dest_file).unwrap();
        let parsed: Template = toml::from_str(&generated_content).unwrap();
        let files = parsed.files.unwrap_or_default();

        assert!(
            !files
                .iter()
                .any(|f| f.path.contains("spark_test_nul_after_1024.bin")),
            "Binary file with NUL past 1024 bytes should be filtered out"
        );
        assert!(
            !files
                .iter()
                .any(|f| f.path.contains("spark_test_invalid_utf8.dat")),
            "Invalid UTF-8 file should be filtered out"
        );
        assert!(
            files
                .iter()
                .any(|f| f.path.contains("spark_test_valid_text.txt")),
            "Valid text file should be included in generated template"
        );
    }

    #[test]
    fn show_info_tolerates_partial_info() {
        let template =
            Template::builder().with_info(Information::new(Some("only-name".into()), None, None));
        // Must not panic.
        Template::show_info(&template);
        let summary = template.info_summary().unwrap();
        assert!(summary.contains("only-name"));
        assert!(!summary.contains("Description"));
        assert!(!summary.contains("Author"));
    }

    #[test]
    fn info_summary_is_none_without_info() {
        assert!(Template::builder().info_summary().is_none());
        let empty = Template::builder().with_info(Information::new(None, None, None));
        assert!(empty.info_summary().is_none());
    }

    #[test]
    fn template_parse_via_fromstr() {
        let template: Template = "[[files]]\npath='a'\ncontent='b'".parse().unwrap();
        assert_eq!(template.files.unwrap()[0].path, "a");
        assert!("[[files".parse::<Template>().is_err());
    }

    #[test]
    fn render_emits_literal_placeholder_for_escaped_token() {
        let template = Template::builder()
            .with_options(Options {
                use_liquid: Some(false),
                ..Options::default()
            })
            .with_file(File::create(
                "{{$$DIR}}/x.txt",
                "keep {{$$NAME:read}} but replace {{$NAME}}",
            ));
        let ctx = Context::new().with_var("NAME", "v").non_interactive();
        let rendered = template.render(&ctx).unwrap();
        assert_eq!(rendered[0].path, "{{$DIR}}/x.txt");
        assert_eq!(rendered[0].content, "keep {{$NAME:read}} but replace v");
    }

    #[test]
    fn metadata_keywords_includes_present_info_and_options() {
        let info = Some(Information::new(
            Some("demo".into()),
            Some("pwn".into()),
            Some("a demo".into()),
        ));
        let options = Options {
            git: true,
            use_liquid: Some(true),
            json_data: None,
            project_root: String::new(),
        };
        let map: HashMap<String, String> = Template::metadata_keywords(&info, &options)
            .into_iter()
            .collect();
        assert_eq!(map.get("{{$INFO_NAME}}").map(String::as_str), Some("demo"));
        assert_eq!(map.get("{{$INFO_AUTHOR}}").map(String::as_str), Some("pwn"));
        assert_eq!(
            map.get("{{$INFO_DESCRIPTION}}").map(String::as_str),
            Some("a demo")
        );
        assert_eq!(
            map.get("{{$OPTIONS_GIT}}").map(String::as_str),
            Some("true")
        );
        assert_eq!(
            map.get("{{$OPTIONS_USE_LIQUID}}").map(String::as_str),
            Some("true")
        );
    }

    #[test]
    fn metadata_keywords_omits_absent_info_fields() {
        let info = Some(Information::new(Some("demo".into()), None, None));
        let pairs = Template::metadata_keywords(&info, &Options::default());
        let keys: Vec<&str> = pairs.iter().map(|(k, _)| k.as_str()).collect();
        assert!(keys.contains(&"{{$INFO_NAME}}"));
        assert!(!keys.contains(&"{{$INFO_AUTHOR}}"));
        assert!(!keys.contains(&"{{$INFO_DESCRIPTION}}"));
        // `[options]` tokens are always present; git defaults to false.
        let map: HashMap<String, String> = pairs.into_iter().collect();
        assert_eq!(
            map.get("{{$OPTIONS_GIT}}").map(String::as_str),
            Some("false")
        );
    }

    #[test]
    fn render_exposes_template_metadata_as_keywords() {
        let template = Template::builder()
            .with_info(Information::new(
                Some("demo".into()),
                Some("pwn".into()),
                None,
            ))
            .with_options(Options {
                git: true,
                use_liquid: Some(false),
                json_data: None,
                project_root: String::new(),
            })
            .with_file(File::create(
                "out.txt",
                "{{$INFO_NAME}} by {{$INFO_AUTHOR}} (git={{$OPTIONS_GIT}})",
            ));
        let ctx = Context::new().non_interactive();
        let rendered = template.render(&ctx).unwrap();
        assert_eq!(rendered[0].content, "demo by pwn (git=true)");
    }

    #[test]
    fn render_lets_supplied_value_override_metadata_keyword() {
        let template = Template::builder()
            .with_info(Information::new(Some("from-info".into()), None, None))
            .with_options(Options {
                use_liquid: Some(false),
                ..Options::default()
            })
            .with_file(File::create("out.txt", "{{$INFO_NAME}}"));
        let ctx = Context::new()
            .with_var("INFO_NAME", "from-cli")
            .non_interactive();
        let rendered = template.render(&ctx).unwrap();
        assert_eq!(rendered[0].content, "from-cli");
    }

    #[test]
    fn render_exposes_template_metadata_as_liquid_globals() {
        let template = Template::builder()
            .with_info(Information::new(Some("demo".into()), None, None))
            .with_options(Options {
                git: true,
                use_liquid: Some(true),
                json_data: None,
                project_root: String::new(),
            })
            .with_file(File::create(
                "out.txt",
                "{% if options.git %}git-on{% else %}git-off{% endif %} {{ info.name | upcase }}",
            ));
        let ctx = Context::new().non_interactive();
        let rendered = template.render(&ctx).unwrap();
        assert_eq!(rendered[0].content, "git-on DEMO");
    }

    #[test]
    fn generate_escapes_placeholders_in_captured_files() {
        let dir_tmp = tempfile::tempdir().unwrap();
        let dir = dir_tmp.path().to_path_buf();
        fs::create_dir_all(&dir).unwrap();
        fs::write(dir.join("tpl.txt"), "hello {{$WORLD}}").unwrap();

        let dest = dir.join("out.toml");
        Template::generate_from(&dir, &dest.to_string_lossy()).unwrap();

        let generated = fs::read_to_string(&dest).unwrap();
        assert!(generated.contains("hello {{$$WORLD}}"), "{generated}");
        let parsed: Template = generated.parse().unwrap();
        assert_eq!(parsed.files.unwrap()[0].path, "tpl.txt");
    }

    #[test]
    fn extract_refuses_to_overwrite_without_force() {
        let dir_tmp = tempfile::tempdir().unwrap();
        let dir = dir_tmp.path().to_path_buf();
        fs::create_dir_all(&dir).unwrap();
        let target = dir.join("README.md");
        fs::write(&target, "precious").unwrap();

        let template = Template::builder().with_file(File::create(
            target.to_string_lossy().to_string(),
            "clobbered",
        ));

        let ctx = Context::new().non_interactive();
        let err = template.extract_with_context(&ctx).unwrap_err();
        assert!(matches!(err, Error::FileExists(ref files) if files.len() == 1));
        assert_eq!(fs::read_to_string(&target).unwrap(), "precious");

        let ctx = Context::new().non_interactive().force(true);
        template.extract_with_context(&ctx).unwrap();
        assert_eq!(fs::read_to_string(&target).unwrap(), "clobbered");
    }

    #[test]
    fn extract_append_mode_merges_into_existing_file_without_force() {
        // The whole point of a patch mode: edit a file that already exists,
        // without --force and without clobbering the original content.
        let dir_tmp = tempfile::tempdir().unwrap();
        let dir = dir_tmp.path().to_path_buf();
        let target = dir.join("CLAUDE.md");
        fs::write(&target, "# Existing rules\n- keep secrets safe\n").unwrap();

        let template = Template::builder().with_file(
            File::create(
                target.to_string_lossy().to_string(),
                "## Added by template\n- always run tests",
            )
            .with_mode(FileMode::Append),
        );

        // No force needed; append is non-destructive.
        template
            .extract_with_context(&Context::new().non_interactive())
            .unwrap();

        assert_eq!(
            fs::read_to_string(&target).unwrap(),
            "# Existing rules\n- keep secrets safe\n## Added by template\n- always run tests"
        );
    }

    #[test]
    fn extract_patch_mode_propagates_unreadable_destination() {
        // A read failure on the destination must NOT be treated as "no content"
        // (that would overwrite the file with just the fragment). Using a
        // directory as the destination is a portable way to force a read error
        // that is not NotFound.
        let dir_tmp = tempfile::tempdir().unwrap();
        let dir = dir_tmp.path().to_path_buf();
        let target = dir.join("occupied");
        fs::create_dir(&target).unwrap();

        let template = Template::builder().with_file(
            File::create(target.to_string_lossy().to_string(), "fragment")
                .with_mode(FileMode::Append),
        );

        let err = template
            .extract_with_context(&Context::new().non_interactive())
            .unwrap_err();
        assert!(
            matches!(err, Error::PatchApply { .. }),
            "expected PatchApply, got {err:?}"
        );
    }

    #[test]
    fn extract_patch_failure_leaves_no_files_written() {
        // Atomicity: a failing patch merge must abort before any earlier file in
        // the same template is written to disk.
        let dir_tmp = tempfile::tempdir().unwrap();
        let dir = dir_tmp.path().to_path_buf();
        let fresh = dir.join("fresh.txt");
        let broken = dir.join("broken.txt");

        let template = Template::builder()
            .with_file(File::create(fresh.to_string_lossy().to_string(), "written"))
            .with_file(
                // insert with no anchor fails during the merge phase.
                File::create(broken.to_string_lossy().to_string(), "x").with_mode(FileMode::Insert),
            );

        let err = template
            .extract_with_context(&Context::new().non_interactive())
            .unwrap_err();
        assert!(matches!(err, Error::PatchApply { .. }), "got {err:?}");
        assert!(
            !fresh.exists(),
            "no file should be written when a later merge fails"
        );
    }

    #[test]
    fn extract_multiple_patches_to_same_file_accumulate() {
        // Two append entries targeting the same path must build on each other,
        // not have the second clobber the first.
        let dir_tmp = tempfile::tempdir().unwrap();
        let dir = dir_tmp.path().to_path_buf();
        let target = dir.join("log.txt");
        fs::write(&target, "base\n").unwrap();

        let path = target.to_string_lossy().to_string();
        let template = Template::builder()
            .with_file(File::create(path.clone(), "one").with_mode(FileMode::Append))
            .with_file(File::create(path, "two").with_mode(FileMode::Append));

        template
            .extract_with_context(&Context::new().non_interactive())
            .unwrap();

        assert_eq!(fs::read_to_string(&target).unwrap(), "base\none\ntwo");
    }

    #[test]
    fn extract_block_mode_is_idempotent_across_runs() {
        let dir_tmp = tempfile::tempdir().unwrap();
        let dir = dir_tmp.path().to_path_buf();
        let target = dir.join("CLAUDE.md");
        fs::write(&target, "# Doc\n").unwrap();

        let template = Template::builder().with_file(
            File::create(target.to_string_lossy().to_string(), "- rule one")
                .with_mode(FileMode::Block)
                .with_marker("rules"),
        );

        let ctx = Context::new().non_interactive();
        template.extract_with_context(&ctx).unwrap();
        // Applying the same template again must not duplicate the block.
        template.extract_with_context(&ctx).unwrap();

        let content = fs::read_to_string(&target).unwrap();
        assert_eq!(content.matches("spark:rules start").count(), 1);
        assert!(content.contains("- rule one"));
    }

    #[test]
    fn extract_insert_mode_resolves_anchor_placeholder() {
        let dir_tmp = tempfile::tempdir().unwrap();
        let dir = dir_tmp.path().to_path_buf();
        let target = dir.join("config.toml");
        fs::write(&target, "[core]\neditor = \"vim\"\n[net]\n").unwrap();

        let template = Template::builder().with_file(
            File::create(target.to_string_lossy().to_string(), "timeout = 30")
                .with_mode(FileMode::Insert)
                .with_anchor("[{{$SECTION}}]")
                .with_position(Position::After),
        );

        let ctx = Context::new().with_var("SECTION", "net").non_interactive();
        template.extract_with_context(&ctx).unwrap();

        assert_eq!(
            fs::read_to_string(&target).unwrap(),
            "[core]\neditor = \"vim\"\n[net]\ntimeout = 30\n"
        );
    }

    #[test]
    fn extract_insert_resolves_placeholder_only_in_anchor() {
        // The anchor references a value that appears nowhere in content/path.
        // It must still be resolved (here from JSON) rather than left literal,
        // which would miss the anchor and silently append at the end.
        let dir_tmp = tempfile::tempdir().unwrap();
        let dir = dir_tmp.path().to_path_buf();
        let target = dir.join("config.toml");
        fs::write(&target, "[core]\n[net]\n").unwrap();

        let template = Template::builder()
            .with_options(Options {
                json_data: Some(serde_json::json!({ "sect": "net" })),
                ..Options::default()
            })
            .with_file(
                File::create(target.to_string_lossy().to_string(), "timeout = 30")
                    .with_mode(FileMode::Insert)
                    .with_anchor("[{{$.sect}}]")
                    .with_position(Position::After),
            );

        template
            .extract_with_context(&Context::new().non_interactive())
            .unwrap();

        assert_eq!(
            fs::read_to_string(&target).unwrap(),
            "[core]\n[net]\ntimeout = 30\n"
        );
    }

    #[test]
    fn extract_insert_anchor_read_errors_non_interactive() {
        // A `:read` placeholder that lives only in the anchor must be surfaced
        // as a missing variable in non-interactive mode — proof the anchor is
        // actually scanned for placeholders.
        let dir_tmp = tempfile::tempdir().unwrap();
        let dir = dir_tmp.path().to_path_buf();
        let target = dir.join("f.txt");
        fs::write(&target, "x\n").unwrap();

        let template = Template::builder().with_file(
            File::create(target.to_string_lossy().to_string(), "y")
                .with_mode(FileMode::Insert)
                .with_anchor("{{$ANCHOR:read}}"),
        );

        let err = template
            .extract_with_context(&Context::new().non_interactive())
            .unwrap_err();
        assert!(
            matches!(err, Error::MissingVariable(ref v) if v == "ANCHOR"),
            "expected MissingVariable(ANCHOR), got {err:?}"
        );
    }

    #[test]
    fn extract_patch_mode_from_toml_applies_unified_diff() {
        let dir_tmp = tempfile::tempdir().unwrap();
        let dir = dir_tmp.path().to_path_buf();
        let target = dir.join("VERSION");
        fs::write(&target, "version = 1\n").unwrap();

        let diff = "--- a\n+++ b\n@@ -1 +1 @@\n-version = 1\n+version = 2\n";
        let template = Template::builder().with_file(
            File::create(target.to_string_lossy().to_string(), diff).with_mode(FileMode::Patch),
        );

        template
            .extract_with_context(&Context::new().non_interactive())
            .unwrap();

        assert_eq!(fs::read_to_string(&target).unwrap(), "version = 2\n");
    }

    #[test]
    fn extract_patch_mode_parses_from_toml_template() {
        // Confirms the `mode` field deserializes from a real TOML template.
        let dir_tmp = tempfile::tempdir().unwrap();
        let dir = dir_tmp.path().to_path_buf();
        let target = dir.join("notes.txt");
        fs::write(&target, "first\n").unwrap();

        let toml_str = format!(
            "[[files]]\npath = \"{}\"\nmode = \"append\"\ncontent = \"second\"\n",
            target.to_string_lossy().replace('\\', "/")
        );
        let template: Template = toml::from_str(&toml_str).unwrap();
        assert_eq!(template.files.as_ref().unwrap()[0].mode(), FileMode::Append);

        template
            .extract_with_context(&Context::new().non_interactive())
            .unwrap();
        assert_eq!(fs::read_to_string(&target).unwrap(), "first\nsecond");
    }

    #[test]
    fn extract_checks_all_files_before_writing_any() {
        let dir_tmp = tempfile::tempdir().unwrap();
        let dir = dir_tmp.path().to_path_buf();
        fs::create_dir_all(&dir).unwrap();
        let fresh = dir.join("fresh.txt");
        let existing = dir.join("existing.txt");
        fs::write(&existing, "keep").unwrap();

        let template = Template::builder()
            .with_file(File::create(fresh.to_string_lossy().to_string(), "new"))
            .with_file(File::create(existing.to_string_lossy().to_string(), "x"));

        let ctx = Context::new().non_interactive();
        assert!(template.extract_with_context(&ctx).is_err());
        assert!(
            !fresh.exists(),
            "no file should be written when the run is refused"
        );
    }

    #[test]
    fn extract_rejects_parent_dir_paths_unless_allowed() {
        let dir_tmp = tempfile::tempdir().unwrap();
        let dir = dir_tmp.path().to_path_buf();
        fs::create_dir_all(dir.join("inner")).unwrap();
        let path = format!("{}/inner/../escaped.txt", dir.display());

        let template = Template::builder().with_file(File::create(path, "x"));

        let err = template
            .extract_with_context(&Context::new().non_interactive())
            .unwrap_err();
        assert!(matches!(err, Error::UnsafePath(_)));
        assert!(!dir.join("escaped.txt").exists());

        template
            .extract_with_context(&Context::new().non_interactive().allow_outside(true))
            .unwrap();
        assert!(dir.join("escaped.txt").exists());
    }

    #[test]
    fn git_init_uses_resolved_project_root() {
        // Regression: `project_root = "{{$PROJECTNAME}}"` with `git = true` used to run
        // the git hook on the raw token, so `git init` failed with
        // "Project root '{{$PROJECTNAME}}' is not a directory".
        let tmp = tempfile::tempdir().unwrap();
        let dir = tmp.path();
        let template = Template {
            info: None,
            options: Some(Options {
                git: true,
                use_liquid: Some(false),
                json_data: None,
                project_root: format!("{}/{{{{$PROJECTNAME}}}}", dir.display()),
            }),
            files: Some(vec![File::new(
                format!("{}/{{{{$PROJECTNAME}}}}/plugin/init.lua", dir.display()),
                "require(\"{{$PROJECTNAME}}\")".into(),
            )]),
        };

        let ctx = Context::new()
            .with_var("PROJECTNAME", "ff")
            .non_interactive()
            .allow_outside(true);
        template.extract_with_context(&ctx).unwrap();

        assert!(dir.join("ff/plugin/init.lua").exists());
        assert!(
            dir.join("ff/.git").is_dir(),
            "git repository should be initialised inside the resolved project root"
        );
        assert!(!dir.join("{{$PROJECTNAME}}").exists());
    }
}
