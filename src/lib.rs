mod parse;
mod gen;

pub use crate::parse::{parse_file, FileData};
pub use crate::gen::component::gen_component;

pub struct Config {
    pub export_func_name: String,
    pub export_mem_name: String
}