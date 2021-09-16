use crate::Word;

pub struct Compiler;

impl Compiler {
    pub fn new() -> Compiler {
        Compiler
    }
    pub fn enter_scope(&mut self, name: &str) {}
    pub fn exit_scope(&mut self) {}
    pub fn finish_current_definition_and_start_next(&mut self) {}
    pub fn exec_word(&mut self, word: &dyn Word) {}
}
