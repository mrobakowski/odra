#![feature(destructuring_assignment)]
#![feature(try_blocks)]

mod vm;

use color_eyre::Result;
use vm::Vm;

fn main() -> Result<()> {
    color_eyre::install()?;

    let mut vm = vm::default_vm();
    vm.run("JSON.stringif({foo: 'bar'})")?;

    Ok(())
}
