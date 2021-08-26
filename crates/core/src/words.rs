use linkme::distributed_slice;
use macros::odra_word;

use crate::compiler::Compiler;
use std::fmt::Debug;

pub trait Word: Sync + Send {
    fn exec(&self, compiler: &mut dyn Compiler);
    fn name(&self) -> &str;
    fn is_macro(&self) -> bool;
    fn stack_effect(&self) -> StackEffect;
}

impl Debug for dyn Word {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct(self.name())
            .field("is_macro", &self.is_macro())
            .field("stack_effect", &self.stack_effect())
            .finish()
    }
}

#[derive(Clone, Debug)]
pub enum StackEffect {
    Dynamic,
    Static {
        inputs: Vec<String>,
        outputs: Vec<String>,
    },
}

#[distributed_slice]
pub static WORDS: [&'static dyn Word] = [..];

#[odra_word]
fn add(a: i64, b: i64) -> i64 {
    a + b
}
