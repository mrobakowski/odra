use color_eyre::Result;
use linkme::distributed_slice;
use macros::odra_macro;
use macros::odra_word;
use std::fmt::Debug;

use crate::OdraType;
use crate::Vm;

pub trait Word: Sync + Send {
    fn exec(&'static self, vm: &mut Vm);
    fn real_exec(&self, vm: &mut Vm);
    fn name(&self) -> &str;
    fn is_macro(&self) -> bool;
    fn stack_effect(&self) -> StackEffect;
    fn register(&self, vm: &mut Vm) -> Result<()>;
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
        inputs: Vec<OdraType>,
        outputs: Vec<OdraType>,
    },
}

#[distributed_slice]
pub static WORDS: [&'static dyn Word] = [..];
pub fn register_all_builtin_words(vm: &mut Vm) -> Result<()> {
    for word in WORDS {
        word.register(vm)?
    }

    Ok(())
}

#[odra_word]
fn add(a: f64, b: f64) -> f64 {
    a + b
}

#[odra_word]
fn one() -> f64 {
    1.0
}

#[odra_word]
fn two() -> f64 {
    2.0
}

#[odra_word]
fn print_stack(vm: &mut Vm) {
    let stack = &vm.main_fibre.stack;
    println!("{stack:?}");
}

#[odra_macro]
fn run(vm: &mut Vm) {
    vm.run_word_being_built()
}
