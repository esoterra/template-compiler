use wasm_encoder::{
    CanonicalFunctionSection, CanonicalOption, Component, ComponentAliasSection,
    ComponentExportKind, ComponentExportSection, ComponentTypeSection, ComponentValType,
    ExportKind, InstanceSection, ModuleSection, PrimitiveValType,
};

use crate::{parse::FileData, Config};

use super::module::gen_module;

pub fn gen_component(config: &Config, file_data: &FileData) -> Component {
    let mut component = Component::new();

    // Encode the inner module
    let module = gen_module(config, file_data);
    component.section(&ModuleSection(&module));

    let mut instances = InstanceSection::new();
    instances.instantiate(0, vec![]);
    component.section(&instances);

    let mut aliases = ComponentAliasSection::new();
    aliases.core_instance_export(0, ExportKind::Func, &config.export_func_name);
    aliases.core_instance_export(0, ExportKind::Memory, &config.export_mem_name);
    component.section(&aliases);

    let mut types = ComponentTypeSection::new();
    types
        .function()
        .params([] as [(&str, ComponentValType); 0])
        .result(ComponentValType::Primitive(PrimitiveValType::String));
    component.section(&types);

    let mut functions = CanonicalFunctionSection::new();
    functions.lift(0, 0, [CanonicalOption::UTF8, CanonicalOption::Memory(0)]);
    component.section(&functions);

    let mut exports = ComponentExportSection::new();
    exports.export(&config.export_func_name, "", ComponentExportKind::Func, 0);
    component.section(&exports);

    component
}
