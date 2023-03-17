mod gen;
mod parse;
mod tokens;

pub use crate::gen::component::gen_component;
pub use crate::parse::{parse_file, FileData, Node};

pub struct Config {
    pub export_func_name: String,
}
