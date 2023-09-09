use std::collections::HashSet;

use wasm_encoder::{
    DataCountSection, DataSection, Function, Instruction, MemArg, TypeSection, ValType, ComponentTypeSection, PrimitiveValType, ComponentValType, BlockType,
};

use crate::{parse::Node, FileData};

const REALLOC_FUNC_INDEX: u32 = 0;
const MEMORY_INDEX: u32 = 0;

const MAX_FLAT_PARAMS: u32 = 16;

pub struct TemplateGenerator<'source> {
    params: Params<'source>,
    file_data: &'source FileData<'source>,
}

pub struct Params<'source> {
    text_params: Vec<&'source str>,
    cond_params: Vec<&'source str>,
}

impl<'source> Params<'source> {
    pub fn new(contents: &'source Vec<Node<'source>>) -> Self {
        let mut text_params = HashSet::new();
        let mut cond_params = HashSet::new();
        for node in contents {
            Self::collect_params(node, &mut text_params, &mut cond_params);
        }
        let mut text_params: Vec<&str> = text_params.into_iter().collect();
        let mut cond_params: Vec<&str> = cond_params.into_iter().collect();
        text_params.sort();
        cond_params.sort();
        Params {
            text_params,
            cond_params,
        }
    }

    fn collect_params(
        node: &'source Node<'source>,
        text_params: &mut HashSet<&'source str>,
        cond_params: &mut HashSet<&'source str>,
    ) {
        match node {
            Node::Text { .. } => {}
            Node::Parameter { name } => {
                text_params.insert(name.value);
            }
            Node::Conditional {
                if_kwd: _,
                cond_ident,
                contents,
                endif_kwd: _,
            } => {
                cond_params.insert(cond_ident.value);
                for node in contents {
                    Self::collect_params(node, text_params, cond_params);
                }
            }
        }
    }

    pub fn stack_len(&self) -> u32 {
        self.text_stack_len() + (self.cond_params.len() as u32)
    }

    fn text_stack_len(&self) -> u32 {
        2 * (self.text_params.len() as u32)
    }

    fn text_mem_len(&self) -> u32 {
        8 * (self.text_params.len() as u32)
    }

    pub fn must_spill(&self) -> bool {
        self.stack_len() > MAX_FLAT_PARAMS
    }

    // The number of text parameters
    pub fn text_params_len(&self) -> usize {
        self.text_params.len()
    }

    // The index in the parameters of a given text parameter name
    pub fn text_param_index(&self, param: &str) -> usize {
        self.text_params.binary_search(&param).unwrap()
    }

    // The index in the parameters of a given condition parameter name
    pub fn cond_param_index(&self, param: &str) -> usize {
        self.cond_params.binary_search(&param).unwrap()
    }

    pub fn record_type(&self) -> ComponentTypeSection {
        let mut types = ComponentTypeSection::new();
        let converted_names: Vec<String> = self.text_params.iter().map(|param: &&str| snake_to_kebab(param)).collect();
        let text_fields = converted_names.iter().map(|param| {
            (
                param.as_str(),
                ComponentValType::Primitive(PrimitiveValType::String),
            )
        });
        let converted_names: Vec<String> = self.cond_params.iter().map(|param: &&str| snake_to_kebab(param)).collect();
        let cond_fields = converted_names.iter().map(|param| {
            (
                param.as_str(),
                ComponentValType::Primitive(PrimitiveValType::Bool),
            )
        });
        let fields: Vec<(&str, ComponentValType)> = text_fields.chain(cond_fields).collect();
        types.defined_type().record(fields);
        types
    }

    fn gen_push_text_offset(&self, func: &mut Function, text_index: u32) {
        self.gen_push_text_field(func, text_index, 0)
    }

    fn gen_push_text_len(&self, func: &mut Function, text_index: u32) {
        self.gen_push_text_field(func, text_index, 1)
    }
    
    fn gen_push_text_field(&self, func: &mut Function, text_index: u32, field: u32) {
        if self.must_spill() {
            // push params offset
            func.instruction(&Instruction::LocalGet(0));
            // push param index shift
            let shift = (text_index * 8) + (field * 4);
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
            let local_index = 2 * text_index + field;
            func.instruction(&Instruction::LocalGet(local_index));
        }
    }

    fn gen_push_cond(&self, func: &mut Function, cond_index: u32) {
        if self.must_spill() {
            // push params offset
            func.instruction(&Instruction::LocalGet(0));
            // push param index shift
            let shift = self.text_mem_len() + cond_index;
            let shift = shift.try_into().unwrap();
            func.instruction(&Instruction::I32Const(shift));
            // compute the final param index
            func.instruction(&Instruction::I32Add);
            // load the param string offset
            func.instruction(&Instruction::I32Load8U(MemArg {
                offset: 0,
                align: 1,
                memory_index: 0,
            }));
        } else {
            let local_index = self.text_stack_len() + cond_index;
            func.instruction(&Instruction::LocalGet(local_index));
        }
    }
}

impl<'source> TemplateGenerator<'source> {
    pub fn new(params: Params<'source>, file_data: &'source FileData<'source>) -> Self {
        Self { params, file_data }
    }

    pub fn params(&self) -> &Params<'source> {
        &self.params
    }

    fn arguments_len(&self) -> u32 {
        if self.params.must_spill() {
            1
        } else {
            self.params.stack_len()
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

    pub fn gen_core_type(&self, types: &mut TypeSection) {
        let params = vec![ValType::I32; self.arguments_len() as usize];
        let results = vec![ValType::I32];
        types.function(params, results);
    }

    pub fn gen_data(&self) -> (DataCountSection, DataSection) {
        let mut count = 0;
        let mut data = DataSection::new();

        for node in self.file_data.contents.iter() {
            Self::collect_data(node, &mut count, &mut data);
        }

        let count = DataCountSection { count };
        (count, data)
    }

    fn collect_data(node: &Node<'source>, count: &mut u32, data: &mut DataSection) {
        match node {
            Node::Text { index: _, text } => {
                data.passive(text.value.bytes());
                *count += 1;
            }
            Node::Parameter { name: _ } => {}
            Node::Conditional {
                if_kwd: _,
                cond_ident: _,
                contents,
                endif_kwd: _,
            } => {
                for node in contents {
                    Self::collect_data(node, count, data);
                }
            }
        }
    }

    pub fn gen_core_function(&self) -> Function {
        // Local variables
        let locals = vec![(self.locals_len(), ValType::I32)];
        let mut func = Function::new(locals);

        self.gen_calculate_len(&mut func);
        self.gen_allocate_results(&mut func);
        self.gen_init_cursor(&mut func);
        self.gen_write_template(&mut func);

        func.instruction(&Instruction::LocalGet(self.return_area_local()));
        func.instruction(&Instruction::End);
        func
    }

    fn gen_calculate_len(&self, func: &mut Function) {
        self.gen_calculate_sequence_len(func, self.file_data.contents.as_slice());
        // Store the calculated length
        func.instruction(&Instruction::LocalSet(self.result_len_local()));
    }

    fn gen_calculate_sequence_len(&self, func: &mut Function, sequence: &[Node<'source>]) {
        let mut base_length = 0;
        let mut param_counts = vec![0; self.params.text_params_len()];
        let mut prior_exists = false;
        for node in sequence.iter() {
            match node {
                Node::Text { index: _, text } => {
                    base_length += text.value.len() as i32;
                }
                Node::Parameter { name } => {
                    let index = self.params.text_param_index(&name.value);
                    param_counts[index] += 1;
                }
                Node::Conditional {
                    if_kwd: _,
                    cond_ident,
                    contents,
                    endif_kwd: _,
                } => {
                    let cond_index = self.params.cond_param_index(cond_ident.value) as u32;

                    self.params.gen_push_cond(func, cond_index);
                    func.instruction(&Instruction::If(BlockType::Result(ValType::I32)));
                    self.gen_calculate_sequence_len(func, &contents);
                    func.instruction(&Instruction::Else);
                    func.instruction(&Instruction::I32Const(0));
                    func.instruction(&Instruction::End);

                    if prior_exists {
                        func.instruction(&Instruction::I32Add);
                    }

                    prior_exists = true;
                }
            }
        }

        // push the base length
        func.instruction(&Instruction::I32Const(base_length));

        if prior_exists {
            func.instruction(&Instruction::I32Add);
        }

        // accumulate the dynamic part of the length
        for (index, count) in param_counts.iter().enumerate() {
            if *count > 0 {
                // load the length of the parameter
                self.params.gen_push_text_len(func, index as u32);
                // push the count of parameter occurrences
                func.instruction(&Instruction::I32Const(*count));
                // multiple the length by the occurrences
                func.instruction(&Instruction::I32Mul);
                // add this length addition to the total length
                func.instruction(&Instruction::I32Add);
            }
        }
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
        let mem_arg = MemArg {
            offset: 0,
            align: 2,
            memory_index: MEMORY_INDEX,
        };
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
        self.gen_write_sequence_template(func, &self.file_data.contents);
    }

    fn gen_write_sequence_template(&self, func: &mut Function, sequence: &[Node<'source>]) {
        for node in sequence {
            // note both branches end by pushing the cursor shift
            match node {
                Node::Text { index, text } => {
                    self.gen_write_segment(func, *index as u32, text.value.len() as i32);
                }
                Node::Parameter { name } => {
                    let index = self.params.text_param_index(&name.value);
                    self.gen_write_param(func, index as u32);
                }
                Node::Conditional {
                    if_kwd: _,
                    cond_ident,
                    contents,
                    endif_kwd: _,
                } => {
                    let cond_index = self.params.cond_param_index(cond_ident.value) as u32;

                    self.params.gen_push_cond(func, cond_index);
                    func.instruction(&Instruction::If(BlockType::Empty));
                    self.gen_write_sequence_template(func, contents);
                    func.instruction(&Instruction::Else);
                    func.instruction(&Instruction::End);
                }
            }
            
            if matches!(node, Node::Text { .. }) || matches!(node, Node::Parameter { .. }) {
                // push cursor and add to shift
                func.instruction(&Instruction::LocalGet(self.result_cursor_local()));
                func.instruction(&Instruction::I32Add);
                func.instruction(&Instruction::LocalSet(self.result_cursor_local()));
            }
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
        self.params.gen_push_text_offset(func, param_index);
        // push length
        self.params.gen_push_text_len(func, param_index);
        // copy the argument data
        func.instruction(&Instruction::MemoryCopy {
            src_mem: MEMORY_INDEX,
            dst_mem: MEMORY_INDEX,
        });

        // push length
        self.params.gen_push_text_len(func, param_index);
    }
}

fn snake_to_kebab(ident: &str) -> String {
    ident.replace("_", "-")
}