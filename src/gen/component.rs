use wasm_encoder::{
    CanonicalFunctionSection, CanonicalOption, Component, ComponentAliasSection,
    ComponentExportKind, ComponentExportSection, ComponentTypeSection, ComponentValType,
    ExportKind, InstanceSection, ModuleSection, PrimitiveValType,
};

use crate::{parse::FileData, Config};

use super::module::gen_module;

/// Generate a component representing the given file data
pub fn gen_component(config: &Config, file_data: &FileData) -> Component {
    let mut component = Component::new();

    // Encode the inner module
    let module = gen_module(config, file_data);
    component.section(&ModuleSection(&module));

    // Instantiate the module
    let mut instances = InstanceSection::new();
    instances.instantiate(0, vec![]);
    component.section(&instances);

    // Project the function and memory into the component index space
    let mut aliases = ComponentAliasSection::new();
    aliases.core_instance_export(0, ExportKind::Func, &config.export_func_name);
    aliases.core_instance_export(0, ExportKind::Memory, &config.export_mem_name);
    component.section(&aliases);

    // Define the component-level function type
    let mut types = ComponentTypeSection::new();
    types
        .function()
        .params([] as [(&str, ComponentValType); 0])
        .result(ComponentValType::Primitive(PrimitiveValType::String));
    component.section(&types);

    // Define the component-level function
    let mut functions = CanonicalFunctionSection::new();
    functions.lift(0, 0, [CanonicalOption::UTF8, CanonicalOption::Memory(0)]);
    component.section(&functions);

    // Export the component-level function
    let mut exports = ComponentExportSection::new();
    exports.export(&config.export_func_name, "", ComponentExportKind::Func, 0);
    component.section(&exports);

    component
}
