use color_eyre::Result;
use compact_str::CompactStr;
use eyre::{eyre, WrapErr};

use crate::AsOdraValue;
use crate::FromOdraValue;
use crate::OdraValue;
use crate::Word;

pub struct Vm {
    pub main_fibre: Fibre,
    other_fibres: Vec<Fibre>,
    vocabulary: Vocabulary,
    word_builder: WordBuilder,
}

pub struct Fibre {
    name: CompactStr,
    pub stack: im::Vector<OdraValue>,
}

struct WordBuilder {
    thunks: Vec<Box<dyn Fn(&mut Vm)>>,
}

impl Vm {
    pub fn new() -> Vm {
        let main_fibre = Fibre {
            name: "main".into(),
            stack: im::Vector::new(),
        };
        let other_fibres = vec![];
        let vocabulary = Vocabulary::empty();
        let word_builder = WordBuilder { thunks: vec![] };

        Vm {
            main_fibre,
            other_fibres,
            vocabulary,
            word_builder,
        }
    }

    pub fn register(&mut self, word: &'static dyn Word) -> Result<()> {
        self.vocabulary
            .append(word)
            .wrap_err_with(|| format!("registering word `{}`", word.name()))
    }

    #[inline]
    pub fn run(&mut self, unresolved_word: CompactStr) -> Result<()> {
        let word = self.vocabulary.resolve(unresolved_word)?;

        word.exec(self);

        Ok(())
    }

    pub fn pop_from_stack<T: FromOdraValue>(&mut self) -> T {
        // TODO: correct fibre... this method should be on the fibre? run & exec should be on the fibre maybe?
        FromOdraValue::from_odra_value(self.main_fibre.stack.pop_back().expect("stack empty"))
            .expect("conversion failed")
    }

    pub fn push_onto_stack<T: AsOdraValue>(&mut self, value: T) {
        self.main_fibre.stack.push_back(value.as_odra_value());
    }

    pub fn append_thunk(&mut self, f: impl Fn(&mut Vm) + 'static) {
        self.word_builder.thunks.push(Box::new(f))
    }

    pub fn run_word_being_built(&mut self) {
        let thunks = std::mem::take(&mut self.word_builder.thunks);
        for thunk in thunks {
            thunk(self)
        }
    } 
}

use vocabulary::Vocabulary;
mod vocabulary {
    use super::*;
    use std::{
        collections::HashMap,
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
        /// must be unique, TODO: semantic unsafe constructor?
        id: CompactStr,
        parent: VocabWeak,
        words: HashMap<CompactStr, &'static dyn Word>, // TODO: leakage!
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
                .map(|x| *x)
                .ok_or(eyre!("could not resolve the word"))
        }

        pub fn append(&mut self, word: &'static dyn Word) -> Result<()> {
            self.current
                .write()
                .map_err(|_| eyre!("vocab lock poisoned"))?
                .words
                .insert(word.name().into(), word);

            Ok(())
        }
    }
}
