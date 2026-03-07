mod codegen;
mod parse;

pub use crate::codegen::{
    component::gen_component,
    template::{Params, TemplateGenerator},
};
pub use crate::parse::{Node, parse_template};

pub struct Config {
    pub export_func_name: String,
}
