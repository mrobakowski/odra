use crate::Result;
use compact_str::CompactStr;

pub struct InputGobbler {
    readline: rustyline::Editor<()>, // TODO: a variant for files would be useful as well
    tokens: Vec<CompactStr>,             // TODO: optimize allocations
}

impl InputGobbler {
    pub fn new() -> InputGobbler {
        let readline = rustyline::Editor::new();
        let tokens = Vec::with_capacity(128);
        InputGobbler { readline, tokens }
    }

    pub fn next(&mut self) -> Result<Option<CompactStr>> {
        loop {
            if let Some(token) = self.tokens.pop() {
                return Ok(Some(token));
            }

            use rustyline::error::ReadlineError::{Eof, Interrupted};
            // we ran out of tokens, time to gobble some up
            // TODO: this is probably too simplistic / wrong
            match self.readline.readline("odra>") {
                Ok(line) => self.tokens.extend(tokenize(&line)),
                Err(Eof | Interrupted) => return Ok(None),
                Err(e) => Err(e)?
            }
        }
    }
}

// TODO: allocations
// TODO: some tokens should be treated with extra care, ex. "{foo bar}" => ["{", "foo", "bar", "}"]
fn tokenize(input: &str) -> impl Iterator<Item = CompactStr> + '_ {
    input.split_whitespace().map(Into::into)
}
