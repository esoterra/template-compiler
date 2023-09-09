mod gen;
mod parse;
mod tokens;

pub use crate::gen::{component::gen_component, template::{TemplateGenerator, Params}};
pub use crate::parse::{parse_file, FileData, Node, M};

pub struct Config {
    pub export_func_name: String,
}
