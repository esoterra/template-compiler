use wasm_encoder::{
    Alias, CanonicalFunctionSection, CanonicalOption, Component, ComponentAliasSection,
    ComponentExportKind, ComponentExportSection, ComponentSectionId, ComponentTypeSection,
    ComponentValType, ExportKind, InstanceSection, ModuleArg, ModuleSection, PrimitiveValType,
    RawSection,
};

use crate::Config;

use super::{module::gen_module, template::TemplateGenerator};

/// Generate a component representing the given file data
pub fn gen_component(config: &Config, template: &TemplateGenerator) -> Component {
    let mut component = Component::new();

    // Encode the allocator module
    let allocator = gen_allocator();
    let id = ComponentSectionId::CoreModule.into();
    let data = allocator.as_slice();
    component.section(&RawSection { id, data });
    let allocator_module_index = 0;

    // Encode the inner module
    let module = gen_module(config, template);
    component.section(&ModuleSection(&module));
    let inner_module_index = 1;

    // Instantiate the allocator & inner module
    let mut instances = InstanceSection::new();
    instances.instantiate::<Vec<(&str, ModuleArg)>, &str>(allocator_module_index, vec![]);
    instances.instantiate(
        inner_module_index,
        [("allocator", ModuleArg::Instance(allocator_module_index))].into_iter(),
    );
    component.section(&instances);

    // Project the function and memory into the component index space
    let mut aliases = ComponentAliasSection::new();
    aliases.alias(Alias::CoreInstanceExport {
        instance: allocator_module_index,
        kind: ExportKind::Memory,
        name: "memory",
    });
    aliases.alias(Alias::CoreInstanceExport {
        instance: allocator_module_index,
        kind: ExportKind::Func,
        name: "realloc",
    });
    aliases.alias(Alias::CoreInstanceExport {
        instance: inner_module_index,
        kind: ExportKind::Func,
        name: &config.export_func_name,
    });
    component.section(&aliases);

    // Define the component-level argument type
    let types = template.params().record_type();
    let params_type_index = 0;
    component.section(&types);

    // Export the component-level argument type
    let mut exports = ComponentExportSection::new();
    exports.export("params", ComponentExportKind::Type, params_type_index, None);
    let params_export_index = 1;
    component.section(&exports);

    // Define the component-level function type
    let mut types = ComponentTypeSection::new();
    types
        .function()
        .params([("params", ComponentValType::Type(params_export_index))])
        .result(ComponentValType::Primitive(PrimitiveValType::String));
    let apply_type_index = 2;
    component.section(&types);

    // Define the component-level function
    let mut functions = CanonicalFunctionSection::new();
    functions.lift(
        1,
        apply_type_index,
        [
            CanonicalOption::UTF8,
            CanonicalOption::Memory(0),
            CanonicalOption::Realloc(0),
        ],
    );
    component.section(&functions);

    // Export the component-level function
    let mut exports = ComponentExportSection::new();
    exports.export(&config.export_func_name, ComponentExportKind::Func, 0, None);
    component.section(&exports);

    component
}

pub fn gen_allocator() -> Vec<u8> {
    let wat = include_str!("../../allocator.wat");
    wat::parse_str(wat).unwrap()
}
