use std::error::Error;

mod vm;

use vm::Vm;

fn main() -> Result<(), Box<dyn Error>> {
    let mut vm = vm::default_vm();
    vm.run("JSON.stringify({foo: 'bar'})")
}
