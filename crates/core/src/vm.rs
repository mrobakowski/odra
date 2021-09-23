use color_eyre::{Report, Result};
use compact_str::CompactStr;
use deno_core::{
    error::{AnyError, JsError},
    JsRuntime,
};
use serde::{de::DeserializeOwned, Serialize};

use crate::Word;

pub struct Vm {
    runtime: JsRuntime,
    vocabulary: Vocabulary,
}

impl Vm {
    pub fn new() -> Vm {
        let mut runtime = JsRuntime::new(Default::default());
        init_odra_stuff(&mut runtime).expect("Could not init the runtime");
        let vocabulary = Vocabulary;

        Vm {
            runtime,
            vocabulary,
        }
    }

    pub fn register<F, Args>(&mut self, word: &dyn Word, f: F)
    where
        F: Fn<Args> + 'static,
        Args: DeserializeOwned,
        F::Output: Serialize + 'static,
    {
        let op_id = self.runtime.register_op(
            word.name(),
            deno_core::op_sync(move |_, a, _: ()| Ok(f.call(a))),
        );

        self.vocabulary.append(word, vocabulary::Entry::Op(op_id))
    }

    pub fn run(&mut self, unresolved_word: CompactStr) -> Result<()> {
        let word = self.vocabulary.resolve(unresolved_word)?;

        Ok(())
    }
}

fn anyhow_to_eyre(err: AnyError) -> Report {
    let js_error: JsError = err.downcast().unwrap();
    Report::new(js_error)
}

fn init_odra_stuff(runtime: &mut JsRuntime) -> Result<()> {
    runtime
        .execute_script("[odra init]", r##"
            // TODO: we should use some chunked immutable stack instead of a linked list for performance
            globalThis.odra_stack = null;
            function odra_push(stack, value) {
                return { value, next: stack }
            }

            globalThis.compiler = {} // TODO: compiler API, should be equivalent from rust and js sides
        "##)
        .map_err(anyhow_to_eyre)?;
    Ok(())
}

use vocabulary::Vocabulary;
mod vocabulary {
    use super::*;
    pub struct Vocabulary;
    pub enum Entry {
        Op(usize),
    }

    impl Vocabulary {
        pub fn resolve(&self, unresolved_word: CompactStr) -> Result<()> {
            Ok(())
        }

        pub fn append(&mut self, word: &dyn Word, entry: Entry) {}
    }
}
