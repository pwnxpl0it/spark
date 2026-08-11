pub mod funcs;
pub mod keywords;
pub mod output_target;
pub mod templates;
mod utils;
pub use output_target::OutputTarget;
use serde::{Deserialize, Serialize};
pub use templates::Options;

pub struct Keywords {}

#[derive(Debug, Deserialize, Serialize)]
pub struct Information {
    pub name: Option<String>,
    pub author: Option<String>,
    pub description: Option<String>,
}

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct File {
    pub path: String,
    pub content: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Template {
    pub info: Option<Information>,
    pub options: Option<Options>,
    pub files: Option<Vec<File>>,
}

#[derive(Debug, Clone, Copy)]
pub enum Fns {
    Read,
    //Env,
    None,
}

impl File {
    pub fn new(path: String, content: String) -> Self {
        Self { path, content }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn file_new_sets_path_and_content() {
        let file = File::new("src/main.rs".into(), "fn main() {}".into());
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
            files: Some(vec![File::new("a.txt".into(), "b".into())]),
        };

        let encoded = toml::to_string(&template).unwrap();
        let decoded: Template = toml::from_str(&encoded).unwrap();
        assert_eq!(decoded.info.unwrap().name.as_deref(), Some("roundtrip"));
        assert_eq!(decoded.files.unwrap()[0].content, "b");
    }
}
