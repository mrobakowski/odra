use std::collections::VecDeque;

use crate::Result;
use compact_str::CompactString;

pub struct InputGobbler {
    readline: rustyline::Editor<()>, // TODO: a variant for files would be useful as well
    tokens: VecDeque<CompactString>,    // TODO: optimize allocations
}

impl InputGobbler {
    pub fn new() -> InputGobbler {
        let readline = rustyline::Editor::new();
        let tokens = VecDeque::with_capacity(128);
        InputGobbler { readline, tokens }
    }

    pub fn next(&mut self) -> Result<Option<CompactString>> {
        let mut num = 0;
        loop {
            if let Some(token) = self.tokens.pop_front() {
                return Ok(Some(token));
            }

            use rustyline::error::ReadlineError::{Eof, Interrupted};
            // we ran out of tokens, time to gobble some up
            // TODO: this is probably too simplistic / wrong
            match self.readline.readline(&format!("odra:{num}> ")) {
                Ok(line) => self.tokens.extend(tokenize(&line)),
                Err(Eof | Interrupted) => return Ok(None),
                Err(e) => Err(e)?,
            }

            num += 1;
        }
    }

    pub fn push_front(&mut self, words: impl DoubleEndedIterator<Item = CompactString>) {
        for word in words.rev() {
            self.tokens.push_front(word) // TODO: why isn't there a push_front_all or sth???
        }
    }
}

// TODO: allocations
// TODO: some tokens should be treated with extra care, ex. "{foo bar}" => ["{", "foo", "bar", "}"]
fn tokenize(input: &str) -> impl Iterator<Item = CompactString> + '_ {
    input.split_whitespace().map(Into::into)
}
