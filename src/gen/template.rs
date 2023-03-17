use wasm_encoder::{Function, Instruction, MemArg, TypeSection, ValType};

use crate::{parse::Node, FileData};

const REALLOC_FUNC_INDEX: u32 = 0;
const MEMORY_INDEX: u32 = 0;

const MAX_FLAT_PARAMS: u32 = 16;

pub struct TemplateGenerator<'source> {
    params: &'source Vec<&'source str>,
    file_data: &'source FileData<'source>,
}

impl<'source> TemplateGenerator<'source> {
    pub fn new(params: &'source Vec<&'source str>, file_data: &'source FileData<'source>) -> Self {
        Self { params, file_data }
    }

    fn params_record_len(&self) -> u32 {
        2 * self.params.len() as u32
    }

    fn params_spill(&self) -> bool {
        self.params_record_len() > MAX_FLAT_PARAMS
    }

    fn arguments_len(&self) -> u32 {
        if self.params_spill() {
            1
        } else {
            self.params_record_len()
        }
    }

    fn result_len_local(&self) -> u32 {
        self.arguments_len() + 0
    }

    fn result_addr_local(&self) -> u32 {
        self.arguments_len() + 1
    }

    fn return_area_local(&self) -> u32 {
        self.arguments_len() + 2
    }

    fn result_cursor_local(&self) -> u32 {
        self.arguments_len() + 3
    }

    fn locals_len(&self) -> u32 {
        4
    }

    pub fn gen_type(&self, types: &mut TypeSection) {
        let params = vec![ValType::I32; self.arguments_len() as usize];
        let results = vec![ValType::I32];
        types.function(params, results);
    }

    pub fn gen_function(&self) -> Function {
        // Local variables
        let locals = vec![(self.locals_len(), ValType::I32)];
        let mut func = Function::new(locals);

        self.gen_calculate_length(&mut func);
        self.gen_allocate_results(&mut func);
        self.gen_init_cursor(&mut func);
        self.gen_write_template(&mut func);

        func.instruction(&Instruction::LocalGet(self.return_area_local()));
        func.instruction(&Instruction::End);
        func
    }

    fn gen_calculate_length(&self, func: &mut Function) {
        let mut base_length = 0;
        let mut counts = vec![0; self.params.len()];
        for (_span, node) in self.file_data.contents.iter() {
            match node {
                Node::Text { text } => {
                    base_length += text.len() as i32;
                }
                Node::Parameter { name } => {
                    let index = self.params.binary_search(name).unwrap();
                    counts[index] += 1;
                }
            }
        }

        // push the base length
        func.instruction(&Instruction::I32Const(base_length));
        // accumulate the dynamic part of the length
        for (index, count) in counts.iter().enumerate() {
            if *count > 0 {
                // load the length of the parameter
                self.gen_push_param_field(func, index as u32, 1);
                // push the count of parameter occurrences
                func.instruction(&Instruction::I32Const(*count));
                // multiple the length by the occurrences
                func.instruction(&Instruction::I32Mul);
                // add this length addition to the total length
                func.instruction(&Instruction::I32Add);
            }
        }
        // Store the calculated length
        func.instruction(&Instruction::LocalSet(self.result_len_local()));
    }

    fn gen_allocate_results(&self, func: &mut Function) {
        // allocate result string
        func.instruction(&Instruction::I32Const(0));
        func.instruction(&Instruction::I32Const(0));
        func.instruction(&Instruction::I32Const(1));
        func.instruction(&Instruction::LocalGet(self.result_len_local()));
        func.instruction(&Instruction::Call(REALLOC_FUNC_INDEX));
        // store allocated address
        func.instruction(&Instruction::LocalSet(self.result_addr_local()));

        // allocate return area
        func.instruction(&Instruction::I32Const(0));
        func.instruction(&Instruction::I32Const(0));
        func.instruction(&Instruction::I32Const(4));
        func.instruction(&Instruction::I32Const(8));
        func.instruction(&Instruction::Call(REALLOC_FUNC_INDEX));
        // store allocated address
        func.instruction(&Instruction::LocalSet(self.return_area_local()));

        // populate return area
        // store result addr
        let mem_arg = MemArg { offset: 0, align: 2, memory_index: MEMORY_INDEX };
        func.instruction(&Instruction::LocalGet(self.return_area_local()));
        func.instruction(&Instruction::LocalGet(self.result_addr_local()));
        func.instruction(&Instruction::I32Store(mem_arg));
        // store result len
        func.instruction(&Instruction::LocalGet(self.return_area_local()));
        func.instruction(&Instruction::I32Const(4));
        func.instruction(&Instruction::I32Add);
        func.instruction(&Instruction::LocalGet(self.result_len_local()));
        func.instruction(&Instruction::I32Store(mem_arg));
    }

    fn gen_init_cursor(&self, func: &mut Function) {
        // set cursor to result string address
        func.instruction(&Instruction::LocalGet(self.result_addr_local()));
        func.instruction(&Instruction::LocalSet(self.result_cursor_local()));
    }

    fn gen_write_template(&self, func: &mut Function) {
        let mut i = 0;
        for (_span, node) in self.file_data.contents.iter() {
            // note both branches end by pushing the cursor shift
            match node {
                Node::Text { text } => {
                    self.gen_write_segment(func, i, text.len() as i32);
                    i += 1;
                }
                Node::Parameter { name } => {
                    let index = self.params.binary_search(name).unwrap();
                    self.gen_write_param(func, index as u32);
                }
            }
            // push cursor and add to shift
            func.instruction(&Instruction::LocalGet(self.result_cursor_local()));
            func.instruction(&Instruction::I32Add);
            func.instruction(&Instruction::LocalSet(self.result_cursor_local()));
        }
    }

    fn gen_write_segment(&self, func: &mut Function, data_index: u32, length: i32) {
        // push destination
        func.instruction(&Instruction::LocalGet(self.result_cursor_local()));
        // push source
        func.instruction(&Instruction::I32Const(0));
        // push length
        func.instruction(&Instruction::I32Const(length));
        // copy data segment into output
        func.instruction(&Instruction::MemoryInit { mem: 0, data_index });

        // push length
        func.instruction(&Instruction::I32Const(length));
    }

    fn gen_write_param(&self, func: &mut Function, param_index: u32) {
        // push destination
        func.instruction(&Instruction::LocalGet(self.result_cursor_local()));
        // push source
        self.gen_push_param_field(func, param_index, 0);
        // push length
        self.gen_push_param_field(func, param_index, 1);
        // copy the argument data
        func.instruction(&Instruction::MemoryCopy {
            src_mem: MEMORY_INDEX,
            dst_mem: MEMORY_INDEX,
        });

        // push length
        self.gen_push_param_field(func, param_index, 1);
    }

    fn gen_push_param_field(&self, func: &mut Function, param_index: u32, field: u32) {
        if self.params_spill() {
            // push params offset
            func.instruction(&Instruction::LocalGet(0));
            // push param index shift
            let shift = (param_index * 8) + (field * 4);
            let shift = shift.try_into().unwrap();
            func.instruction(&Instruction::I32Const(shift));
            // compute the final param index
            func.instruction(&Instruction::I32Add);
            // load the param string offset
            func.instruction(&Instruction::I32Load(MemArg {
                offset: 0,
                align: 4,
                memory_index: 0,
            }));
        } else {
            let local_index = 2 * param_index + field;
            func.instruction(&Instruction::LocalGet(local_index));
        }
    }
}
