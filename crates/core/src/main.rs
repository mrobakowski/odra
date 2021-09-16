#![feature(destructuring_assignment)]
#![feature(try_blocks)]

mod compiler;
mod vm;
mod words;
mod input_gobbler;

use color_eyre::Result;
use rustyline::Editor;

pub use compiler::Compiler;
pub use vm::Vm;
pub use words::StackEffect;
pub use words::Word;
pub use input_gobbler::InputGobbler;

pub type BCode = [BCodeInstruction];

pub enum BCodeInstruction {
    /// LEAKY_ABSTRACTION: the string is a js expression for now
    Push(String),
    OdraCall(String), // TODO: those strings in the calls kinda suck, maybe they could be Global<Value>s (LEAKY_ABSTRACTION) and then simply invoked
    JsCall { fn_name: String, args: Vec<String> },
}

fn main() -> Result<()> {
    color_eyre::install()?;

    println!("WORDS: {:#?}", &words::WORDS[..]);

    let mut vm = vm::default_vm();
    let mut compiler = Compiler::new();
    let mut rl = Editor::<()>::new();

    'main_loop: loop {
        match rl.readline("odra>") {
            Ok(line) => todo!(),
            Err(_) => {
                print!("some kind of readline error, aborting");
                break 'main_loop;
            }
        }
    }

    Ok(())
}
