use linkme::distributed_slice;
use macros::odra_word;

use std::fmt::Debug;

use crate::Vm;

pub trait Word: Sync + Send {
    fn exec(&self, vm: &mut Vm);
    fn name(&self) -> &str;
    fn is_macro(&self) -> bool;
    fn stack_effect(&self) -> StackEffect;
    fn register(&self, vm: &mut Vm);
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
pub fn register_all_builtin_words(vm: &mut Vm) {
    for word in WORDS {
        word.register(vm)
    }
}

#[odra_word]
fn add(a: f64, b: f64) -> f64 {
    a + b
}
