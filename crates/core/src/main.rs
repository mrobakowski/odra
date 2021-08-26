#![feature(destructuring_assignment)]
#![feature(try_blocks)]

mod vm;
mod compiler;
mod words;

use color_eyre::Result;

pub use vm::Vm;
pub use words::Word;
pub use words::StackEffect;
pub use compiler::Compiler;

pub type BCode = str;

fn main() -> Result<()> {
    color_eyre::install()?;

    println!("WORDS: {:#?}", &words::WORDS[..]);

    let mut vm = vm::default_vm();
    vm.run("JSON.stringify({foo: 'bar'})")?;

    Ok(())
}
