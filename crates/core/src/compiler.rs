
use crate::Word;

pub trait Compiler {
    fn enter_scope(&mut self, name: &str);
    fn exit_scope(&mut self);
    fn finish_current_definition_and_start_next(&mut self);
    fn exec_word(&mut self, word: &dyn Word);
}
