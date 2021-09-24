use color_eyre::{Report, Result};
use compact_str::CompactStr;
use deno_core::{
    error::{AnyError, JsError},
    JsRuntime,
};
use eyre::eyre;
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
        let vocabulary = Vocabulary::empty();

        Vm {
            runtime,
            vocabulary,
        }
    }

    pub fn register<F, Args>(&mut self, word: &'static dyn Word, f: F) -> Result<()>
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

        word.exec(self);

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
    use std::{
        collections::{HashMap},
        hash::{Hash, Hasher},
        sync::{Arc, RwLock, Weak},
    };

    // TODO: can we get away without Arcs here?
    type VocabRef = Arc<RwLock<VocabInner>>;
    type VocabWeak = Weak<RwLock<VocabInner>>;
    pub struct Vocabulary {
        all_vocabs: HashMap<CompactStr, VocabRef>,
        current: VocabRef,
        root: VocabRef,
    }

    struct VocabInner {
        /// must be unique
        id: CompactStr,
        parent: VocabWeak,
        words: HashMap<CompactStr, (&'static dyn Word, Entry)>,
        children: HashMap<CompactStr, VocabRef>,
    }

    impl Eq for VocabInner {}
    impl PartialEq for VocabInner {
        fn eq(&self, other: &Self) -> bool {
            self.id == other.id
        }
    }

    impl Hash for VocabInner {
        fn hash<H: Hasher>(&self, state: &mut H) {
            self.id.hash(state);
        }
    }

    pub enum Entry {
        Op(usize),
    }

    impl Vocabulary {
        pub fn empty() -> Vocabulary {
            let key: CompactStr = "root".into();
            let root = Arc::new(RwLock::new(VocabInner {
                id: key.clone(),
                parent: Weak::new(),
                words: HashMap::new(),
                children: HashMap::new(),
            }));
            let current = Arc::clone(&root);
            let all_vocabs = HashMap::from([(key, Arc::clone(&root))]);

            Vocabulary {
                all_vocabs,
                current,
                root,
            }
        }

        pub fn resolve(&self, unresolved_word: CompactStr) -> Result<&'static dyn Word> {
            self.current
                .read()
                .map_err(|_| eyre!("vocab lock poisoned"))?
                .words
                .get(&unresolved_word)
                .map(|(word, _)| *word)
                .ok_or(eyre!("could not resolve the word"))
        }

        pub fn append(&mut self, word: &'static dyn Word, entry: Entry) -> Result<()> {
            self.current
                .write()
                .map_err(|_| eyre!("vocab lock poisoned"))?
                .words
                .insert(word.name().into(), (word, entry));

            Ok(())
        }
    }
}
