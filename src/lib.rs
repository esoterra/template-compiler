mod gen;
mod parse;

pub use crate::gen::component::gen_component;
pub use crate::parse::{parse_file, FileData};

pub struct Config {
    pub export_func_name: String,
    pub export_mem_name: String,
}
