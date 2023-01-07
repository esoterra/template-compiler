use wasm_encoder::{
    CodeSection, ConstExpr, DataSection, ExportKind, ExportSection, Function, FunctionSection,
    Instruction, MemorySection, MemoryType, Module, TypeSection, ValType,
};

use crate::{parse::FileData, Config};

pub fn gen_module(config: &Config, file_data: &FileData) -> Module {
    let layout = calculate_layout(file_data);

    // Create a type entry for the `apply` function's type
    let TypeInfo { types, type_index } = gen_types();
    // Create a function entry for the `apply` function
    let FunctionInfo {
        functions,
        function_index,
    } = gen_functions(type_index);
    // Generate a memory section with enough room for the static data
    let MemoryInfo {
        memories,
        memory_index,
    } = gen_memories(&layout);
    // Generate a code section that returns a pointer into the return area
    let codes = gen_codes(&layout);
    // Generate a data section with the static data
    let data = gen_data(file_data, memory_index, &layout);

    // Create an export entry for the `apply` function
    let exports = gen_exports(&config, function_index, memory_index);

    // Construct a module in the required order
    let mut module = Module::new();
    module.section(&types);
    module.section(&functions);
    module.section(&memories);
    module.section(&exports);
    module.section(&codes);
    module.section(&data);

    // Return the constructed module and export index for the `apply` function
    module
}

struct MemoryLayout {
    return_area_offset: u32,
    template_data_offset: u32,
    pages_needed: u64,
}

fn calculate_layout(file_data: &FileData) -> MemoryLayout {
    let return_area_offset: u32 = 0;
    let return_area_len: u64 = 8;

    let template_data_offset: u32 = return_area_len as u32;
    let template_data_len: u64 = file_data.contents.len() as u64;

    let memory_needed = return_area_len + template_data_len;

    const PAGE_SIZE: u64 = 1 << 16;
    let pages_needed = div_ceil(memory_needed, PAGE_SIZE);

    MemoryLayout {
        return_area_offset,
        template_data_offset,
        pages_needed,
    }
}

struct TypeInfo {
    types: TypeSection,
    type_index: u32,
}

fn gen_types() -> TypeInfo {
    let mut types = TypeSection::new();
    let params = vec![];
    let results = vec![ValType::I32];
    types.function(params, results);

    TypeInfo {
        types,
        type_index: 0,
    }
}

struct FunctionInfo {
    functions: FunctionSection,
    function_index: u32,
}

fn gen_functions(type_index: u32) -> FunctionInfo {
    let mut functions = FunctionSection::new();
    functions.function(type_index);
    FunctionInfo {
        functions,
        function_index: 0,
    }
}

fn gen_exports(config: &Config, function_index: u32, memory_index: u32) -> ExportSection {
    let mut exports = ExportSection::new();
    exports.export(&config.export_func_name, ExportKind::Func, function_index);
    exports.export(&config.export_mem_name, ExportKind::Memory, memory_index);
    exports
}

struct MemoryInfo {
    memories: MemorySection,
    memory_index: u32,
}

fn gen_memories(layout: &MemoryLayout) -> MemoryInfo {
    let mut memories = MemorySection::new();
    memories.memory(MemoryType {
        minimum: layout.pages_needed,
        maximum: None,
        memory64: false,
        shared: false,
    });
    MemoryInfo {
        memories,
        memory_index: 0,
    }
}

fn gen_codes(layout: &MemoryLayout) -> CodeSection {
    let mut codes = CodeSection::new();
    let locals = vec![];
    let mut func = Function::new(locals);
    func.instruction(&Instruction::I32Const(layout.return_area_offset as i32));
    func.instruction(&Instruction::End);
    codes.function(&func);
    codes
}

fn gen_data(file_data: &FileData, memory_index: u32, layout: &MemoryLayout) -> DataSection {
    let mut data = DataSection::new();

    let result_start: u32 = 8;
    let result_len = file_data.contents.len() as u32;
    let return_area = [result_start.to_le_bytes(), result_len.to_le_bytes()].concat();
    data.active(
        memory_index,
        &ConstExpr::i32_const(layout.return_area_offset as i32),
        return_area,
    );

    data.active(
        memory_index,
        &ConstExpr::i32_const(layout.template_data_offset as i32),
        file_data.contents.bytes(),
    );

    data
}

fn div_ceil(a: u64, b: u64) -> u64 {
    (a + b - 1) / b
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_div_ceil() {
        assert_eq!(div_ceil(6, 5), 2);
        assert_eq!(div_ceil(5, 5), 1);
        assert_eq!(div_ceil(4, 5), 1);
    }
}
