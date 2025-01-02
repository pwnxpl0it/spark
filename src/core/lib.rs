pub mod funcs;
pub mod keywords;
mod templates;
mod utils;
use serde::{Deserialize, Serialize};
pub struct Keywords {}
pub use templates::Options;

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
