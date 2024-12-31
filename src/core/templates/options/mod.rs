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
