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

    pub struct Deno {
        runtime: JsRuntime,
    }

    impl Deno {
        pub fn new() -> Deno {
            let runtime = JsRuntime::new(Default::default());
            Deno { runtime }
        }
    }

    impl Vm for Deno {
        fn run(&mut self, code: impl AsRef<BCode>) -> Result<()> {
            let code = code.as_ref();
            let res = self
                .runtime
                .execute_script("[odra]", code)
                .map_err(anyhow_to_eyre)?;
            let res: &Value = res.borrow();

            println!(
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
}
