use color_eyre::{ Result};
use compact_str::CompactStr;
use eyre::{eyre, WrapErr};
use serde::{de::DeserializeOwned, Serialize};

use crate::OdraValue;
use crate::Word;

pub struct Vm {
    main_fibre: Fibre,
    other_fibres: Vec<Fibre>,
    vocabulary: Vocabulary,
}

struct Fibre {
    name: CompactStr,
    stack: im::Vector<OdraValue>,
}

impl Vm {
    pub fn new() -> Vm {
        let main_fibre = Fibre {
            name: "main".into(),
            stack: im::Vector::new(),
        };
        let other_fibres = vec![];
        let vocabulary = Vocabulary::empty();

        Vm {
            main_fibre,
            other_fibres,
            vocabulary,
        }
    }

    pub fn register<F, Args>(&mut self, word: &'static dyn Word, f: F) -> Result<()>
    where
        F: Fn<Args> + 'static,
        Args: DeserializeOwned,
        F::Output: Serialize + 'static,
    {
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
        /// must be unique
        id: CompactStr,
        parent: VocabWeak,
        words: HashMap<CompactStr, &'static dyn Word>,
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
