mod codegen;
mod parse;
mod tokens;

pub use crate::codegen::{component::gen_component, template::{TemplateGenerator, Params}};
pub use crate::parse::{parse_file, FileData, Node, M};

pub struct Config {
    pub export_func_name: String,
}
