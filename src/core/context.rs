use crate::error::Result;
use crate::Keywords;
use std::collections::HashMap;

/// Execution context for resolving placeholders, JSON paths, and managing interactivity.
#[derive(Debug, Clone)]
pub struct Context {
    pub(crate) keywords: HashMap<String, String>,
    pub(crate) json_data: Option<serde_json::Value>,
    pub(crate) interactive: bool,
}

impl Default for Context {
    fn default() -> Self {
        Self::init()
    }
}

impl Context {
    /// Creates a blank context without initializing built-in date/system variables.
    pub fn new() -> Self {
        Self {
            keywords: HashMap::new(),
            json_data: None,
            interactive: true,
        }
    }

    /// Initializes a context pre-populated with standard built-in variables
    /// (`{{$NOW}}`, `{{$YYYY}}`, `{{$HOME}}`, environment variables, etc.).
    pub fn init() -> Self {
        Self {
            keywords: Keywords::init(),
            json_data: None,
            interactive: true,
        }
    }

    /// Sets a template variable. If `key` is not already enclosed in `{{$...}}`,
    /// it will be formatted automatically.
    ///
    /// # Examples
    /// ```rust
    /// use spark::Context;
    ///
    /// let ctx = Context::new()
    ///     .with_var("NAME", "Alice")
    ///     .with_var("{{$ROLE}}", "Admin");
    ///
    /// assert_eq!(ctx.get_var("NAME"), Some("Alice"));
    /// assert_eq!(ctx.get_var("ROLE"), Some("Admin"));
    /// ```
    pub fn with_var(mut self, key: impl AsRef<str>, val: impl Into<String>) -> Self {
        self.set_var(key.as_ref(), val.into());
        self
    }

    /// Inserts or updates a template variable.
    pub fn set_var(&mut self, key: &str, val: impl Into<String>) {
        let val_str = val.into();
        let formatted_key = if key.starts_with("{{$") && key.ends_with("}}") {
            key.to_string()
        } else {
            Keywords::from(key, None)
        };
        self.keywords.insert(formatted_key, val_str);
    }

    /// Extends the context with multiple key-value variable pairs.
    ///
    /// Accepts any iterator of pairs, including a [`HashMap`]. Bare keys like
    /// `"NAME"` are formatted as `{{$NAME}}` automatically — the same as
    /// [`Context::with_var`].
    ///
    /// Prefer this over [`Context::from`] with a [`HashMap`]. `From` copies keys
    /// as-is, so they must already be in `{{$NAME}}` form or placeholders will
    /// not be substituted.
    ///
    /// # Examples
    /// ```rust
    /// use std::collections::HashMap;
    /// use spark::Context;
    ///
    /// let mut vars = HashMap::new();
    /// vars.insert("NAME", "Alice");
    /// vars.insert("ROLE", "Admin");
    ///
    /// let ctx = Context::new().with_vars(vars);
    ///
    /// assert_eq!(ctx.get_var("NAME"), Some("Alice"));
    /// assert_eq!(ctx.get_var("ROLE"), Some("Admin"));
    /// ```
    ///
    /// Arrays work the same way:
    ///
    /// ```rust
    /// use spark::Context;
    ///
    /// let ctx = Context::new().with_vars([
    ///     ("NAME", "Alice"),
    ///     ("ROLE", "Admin"),
    /// ]);
    ///
    /// assert_eq!(ctx.get_var("NAME"), Some("Alice"));
    /// ```
    pub fn with_vars<I, K, V>(mut self, vars: I) -> Self
    where
        I: IntoIterator<Item = (K, V)>,
        K: AsRef<str>,
        V: Into<String>,
    {
        for (k, v) in vars {
            self.set_var(k.as_ref(), v);
        }
        self
    }

    /// Attaches JSON data for resolving `{{$.path.to.key}}` placeholders.
    pub fn with_json(mut self, json: serde_json::Value) -> Self {
        self.json_data = Some(json);
        self
    }

    /// Parses and attaches a JSON string for resolving `{{$.path.to.key}}` placeholders.
    pub fn with_json_str(self, json_str: &str) -> Result<Self> {
        let value: serde_json::Value = serde_json::from_str(json_str)?;
        Ok(self.with_json(value))
    }

    /// Configures whether interactive terminal prompts (like `:read` or missing `PROJECTNAME`)
    /// are permitted. When set to `false`, missing variables will immediately return
    /// [`crate::Error::MissingVariable`] instead of blocking `stdin`.
    pub fn with_interactive(mut self, interactive: bool) -> Self {
        self.interactive = interactive;
        self
    }

    /// Disables interactive prompts.
    pub fn non_interactive(mut self) -> Self {
        self.interactive = false;
        self
    }

    /// Checks if interactive prompts are enabled.
    pub fn is_interactive(&self) -> bool {
        self.interactive
    }

    /// Returns a reference to the internal keywords map.
    pub fn keywords(&self) -> &HashMap<String, String> {
        &self.keywords
    }

    /// Returns a mutable reference to the internal keywords map.
    pub fn keywords_mut(&mut self) -> &mut HashMap<String, String> {
        &mut self.keywords
    }

    /// Returns a reference to the attached JSON data, if any.
    pub fn json_data(&self) -> Option<&serde_json::Value> {
        self.json_data.as_ref()
    }

    /// Looks up the value of a variable.
    pub fn get_var(&self, key: &str) -> Option<&str> {
        if let Some(v) = self.keywords.get(key) {
            return Some(v.as_str());
        }
        let formatted = Keywords::from(key, None);
        self.keywords.get(&formatted).map(|s| s.as_str())
    }
}

impl From<HashMap<String, String>> for Context {
    fn from(keywords: HashMap<String, String>) -> Self {
        Self {
            keywords,
            json_data: None,
            interactive: true,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn context_with_var_formats_key() {
        let ctx = Context::new().with_var("NAME", "spark");
        assert_eq!(ctx.get_var("NAME"), Some("spark"));
        assert_eq!(ctx.get_var("{{$NAME}}"), Some("spark"));
    }

    #[test]
    fn context_with_json_and_interactivity() {
        let ctx = Context::new()
            .with_json(serde_json::json!({ "foo": "bar" }))
            .non_interactive();

        assert!(!ctx.is_interactive());
        assert_eq!(ctx.json_data().unwrap()["foo"], "bar");
    }
}
