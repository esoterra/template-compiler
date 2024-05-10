use wasm_encoder::{
    CodeSection, EntityType, ExportKind, ExportSection,
    FunctionSection, ImportSection, MemoryType, Module, TypeSection, ValType,
};

use crate::Config;

use super::template::TemplateGenerator;

pub fn gen_module(config: &Config, template: &TemplateGenerator) -> Module {
    // Create a type entry for the `apply` function's type
    let mut types = TypeSection::new();
    types.function(vec![ValType::I32; 4], vec![ValType::I32; 1]);
    let realloc_type_index = 0;
    types.function(vec![], vec![]);
    let clear_type_index = 1;
    template.gen_core_type(&mut types);
    let template_type_index = 2;

    // Create imports for the allocator memory, alloc, and clear
    let mut imports = ImportSection::new();
    let memory_type = MemoryType {
        minimum: 1,
        maximum: None,
        memory64: false,
        shared: false,
        page_size_log2: None,
    };
    let memory_index = 0;
    imports.import("allocator", "memory", EntityType::Memory(memory_type));
    let realloc_func_index = 0;
    imports.import(
        "allocator",
        "realloc",
        EntityType::Function(realloc_type_index),
    );
    let clear_func_index = 1;
    imports.import("allocator", "clear", EntityType::Function(clear_type_index));

    // Create a function entry for the `apply` function
    let mut functions = FunctionSection::new();
    functions.function(template_type_index);
    let template_func_index = 2;

    // Generate a code section that returns a pointer into the return area
    let mut codes = CodeSection::new();
    codes.function(&template.gen_core_function());

    // Generate a data section with the static data
    let (count, data) = template.gen_data();

    // Create an export entry for the `apply` function
    let mut exports = ExportSection::new();
    exports.export("memory", ExportKind::Memory, memory_index);
    exports.export("realloc", ExportKind::Func, realloc_func_index);
    exports.export("clear", ExportKind::Func, clear_func_index);
    exports.export(
        &config.export_func_name,
        ExportKind::Func,
        template_func_index,
    );

    // Construct a module in the required order
    let mut module = Module::new();
    module.section(&types);
    module.section(&imports);
    module.section(&functions);
    module.section(&exports);
    module.section(&count);
    module.section(&codes);
    module.section(&data);

    // Return the constructed module and export index for the `apply` function
    module
}


