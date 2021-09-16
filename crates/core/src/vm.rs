use color_eyre::Result;

use crate::BCode;

pub trait Vm {
    fn run(&mut self, code: impl AsRef<BCode>) -> Result<()>;
}

pub fn default_vm() -> impl Vm {
    deno_vm::Deno::new()
}

mod deno_vm {
    use super::*;
    use color_eyre::Report;
    use deno_core::{
        error::{AnyError, JsError},
        v8::Value,
        JsRuntime,
    };
    use std::borrow::Borrow;
    use std::fmt::Write;

    pub struct Deno {
        runtime: JsRuntime,
    }

    impl Deno {
        pub fn new() -> Deno {
            let mut runtime = JsRuntime::new(Default::default());
            init_odra_stuff(&mut runtime).expect("Could not init the runtime");

            Deno { runtime }
        }
    }

    impl Vm for Deno {
        fn run(&mut self, code: impl AsRef<BCode>) -> Result<()> {
            let code = code.as_ref();

            let js_code = bcode_to_js(code);
            eprintln!("js code:\n{}", js_code);

            let res = self
                .runtime
                .execute_script("[odra]", &js_code)
                .map_err(anyhow_to_eyre)?;

            let res: &Value = res.borrow();

            eprintln!(
                "script result: {}",
                res.to_rust_string_lossy(&mut self.runtime.handle_scope())
            );

            Ok(())
        }
    }

    fn anyhow_to_eyre(err: AnyError) -> Report {
        let js_error: JsError = err.downcast().unwrap();
        Report::new(js_error)
    }

    fn bcode_to_js(code: &BCode) -> String {
        let mut buffer = String::new();

        for instruction in code {
            match instruction {
                crate::BCodeInstruction::Push(value_string) => writeln!(
                    buffer,
                    "globalThis.odra_stack = odra_push(globalThis.odra_stack, {})",
                    value_string
                )
                .unwrap(),

                crate::BCodeInstruction::OdraCall(fn_name) => writeln!(
                    buffer,
                    "globalThis.odra_stack = {}(globalThis.odra_stack)",
                    fn_name
                )
                .unwrap(),

                crate::BCodeInstruction::JsCall { fn_name, num_args } => {
                    for i in 0..*num_args {
                        writeln!(buffer, "let ${} = globalThis.odra_stack.value; globalThis.odra_stack = globalThis.odra_stack.next", i).unwrap();
                    }
                    write!(
                        buffer,
                        "globalThis.odra_stack = odra_push(globalThis.odra_stack, {}(",
                        fn_name
                    )
                    .unwrap();
                    for i in 0..*num_args {
                        write!(buffer, "${},", i).unwrap();
                    }
                    writeln!(buffer, "))").unwrap();
                }
            }
        }

        buffer
    }

    fn init_odra_stuff(runtime: &mut JsRuntime) -> Result<()> {
        runtime
            .execute_script("[odra init]", r##"
                // TODO: we should use some chunked immutable stack instead of a linked list for performance
                globalThis.odra_stack = null;
                function odra_push(stack, value) {
                    return { value, next: stack }
                }

                globalThis.compiler = {} // TODO: compiler API, should be equivalent from rust and js sides
            "##)
            .map_err(anyhow_to_eyre)?;
        Ok(())
    }
}
