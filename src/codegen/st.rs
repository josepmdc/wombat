use std::collections::HashMap;

use inkwell::values::PointerValue;

use super::error::CodegenError;

type Table<'ctx> = HashMap<String, PointerValue<'ctx>>;

pub struct SymbolTable<'ctx> {
    table_stack: Vec<Table<'ctx>>,
}

impl<'ctx> SymbolTable<'ctx> {
    pub fn new() -> Self {
        SymbolTable {
            table_stack: vec![HashMap::new()],
        }
    }

    pub fn new_context(&mut self) {
        self.table_stack.push(HashMap::new());
    }

    pub fn pop_context(&mut self) {
        self.table_stack.pop();
    }

    pub fn top_table(&mut self) -> &mut Table<'ctx> {
        self.table_stack
            .last_mut()
            .expect("There should always be at least one table")
    }

    pub fn insert(&mut self, name: String, value: PointerValue<'ctx>) {
        self.top_table().insert(name, value);
    }

    pub fn get(&self, var: &str) -> Result<PointerValue<'ctx>, CodegenError> {
        let table = self
            .table_stack
            .iter()
            .rev()
            .find(|table| table.contains_key(var))
            .ok_or_else(|| CodegenError::UndeclaredVariableOrOutOfScope(
                var.to_string(),
            ))?;

        Ok(*table.get(var).unwrap())
    }
}
