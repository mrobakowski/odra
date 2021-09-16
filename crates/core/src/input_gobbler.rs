pub struct InputGobbler {
    readline: rustyline::Editor<()>, // TODO: a variant for files would be useful as well
    tokens: Vec<String>,             // TODO: optimize allocations
}

impl InputGobbler {
    pub fn new() -> InputGobbler {
        let readline = rustyline::Editor::new();
        let tokens = Vec::with_capacity(128);
        InputGobbler { readline, tokens }
    }

    pub fn next(&mut self) -> String {
        loop {
            if let Some(token) = self.tokens.pop() {
                return token;
            }

            // we ran out of tokens, time to gobble some up
            match self.readline.readline("odra>") {
                Ok(line) => self.tokens.extend(tokenize(line)),
                Err(_) => todo!("handle readline error"),
            }
        }
    }
}

// TODO: allocations
// TODO: some tokens should be treated with extra care, ex. "{foo bar}" => ["{", "foo", "bar", "}"]
fn tokenize(input: String) -> Vec<String> {
    input.split_whitespace().map(str::to_string).collect()
}
