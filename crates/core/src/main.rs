#![feature(destructuring_assignment)]
#![feature(try_blocks)]
#![feature(unboxed_closures)]
#![feature(fn_traits)]
#![feature(const_type_id)]
#![feature(generic_associated_types)]

mod input_gobbler;
mod spec_get_vm;
mod value;
mod vm;
mod words;

use color_eyre::Result;

pub use input_gobbler::InputGobbler;
pub use value::OdraType;
pub use value::OdraValue;
pub use value::{AsOdraValue, FromOdraValue, AsOdraType};
pub use vm::Vm;
pub use words::StackEffect;
pub use words::Word;

fn main() -> Result<()> {
    color_eyre::install()?;

    println!("WORDS: {:#?}", &words::WORDS[..]);

    let mut vm = Vm::new();
    let mut gobbler = InputGobbler::new();
    words::register_all_builtin_words(&mut vm)?;

    loop {
        match gobbler.next()? {
            Some(word) => vm.run(word)?,
            None => todo!("print stack if interactive session and exit"),
        }
    }
}
