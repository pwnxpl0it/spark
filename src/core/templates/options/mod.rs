use crate::Options;
use colored::*;
pub mod git;

impl Default for Options {
    fn default() -> Self {
        Self {
            json_data: Some(serde_json::Value::Null),
            use_liquid: Some(true),
            git: false,
            project_root: String::new(),
        }
    }
}

impl Options {
    pub fn set_git(&mut self, git: bool) {
        self.git = git;
    }

    pub fn set_json(&mut self, json_data: serde_json::Value) {
        self.json_data = Some(json_data);
    }

    pub fn set_project_root(&mut self, project_root: &str) {
        self.project_root = project_root.to_string();
    }

    pub fn handle(self) {
        if self.git {
            if self.project_root.is_empty() {
                eprintln!(
                    "\n{}: {}",
                    "error".to_string().red(),
                    "Project root is not set".yellow()
                );
                return;
            }

            println!(
                "\nInitializing git repository for {}\n",
                self.project_root.blue()
            );

            git::init(&self.project_root);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_options() {
        let options = Options::default();
        assert!(!options.git);
        assert_eq!(options.use_liquid, Some(true));
        assert_eq!(options.json_data, Some(serde_json::Value::Null));
        assert!(options.project_root.is_empty());
    }

    #[test]
    fn setters_update_fields() {
        let mut options = Options::default();
        options.set_git(true);
        options.set_project_root("my_project");
        options.set_json(serde_json::json!({ "k": "v" }));

        assert!(options.git);
        assert_eq!(options.project_root, "my_project");
        assert_eq!(options.json_data, Some(serde_json::json!({ "k": "v" })));
    }

    #[test]
    fn handle_is_noop_when_git_disabled() {
        let options = Options {
            git: false,
            use_liquid: None,
            json_data: None,
            project_root: String::new(),
        };
        // Should return without attempting git init.
        options.handle();
    }

    #[test]
    fn handle_skips_git_init_when_project_root_missing() {
        let options = Options {
            git: true,
            use_liquid: None,
            json_data: None,
            project_root: String::new(),
        };
        // Should print an error and return without panicking.
        options.handle();
    }
}
