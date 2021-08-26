use color_eyre::{Report, Result};
use core::fmt::Debug;
use deno_core::{self, JsRuntime, error::{AnyError, JsError}, v8::Value};
use std::borrow::Borrow;

pub(crate) trait Vm {
    type BCode: Debug + ?Sized;
    fn run(&mut self, code: impl AsRef<Self::BCode>) -> Result<()>;
}

pub(crate) fn default_vm() -> impl Vm<BCode = str> {
    Deno::new()
}

struct Deno {
    runtime: JsRuntime,
}

impl Deno {
    fn new() -> Deno {
        let runtime = JsRuntime::new(Default::default());
        Deno { runtime }
    }
}

impl Vm for Deno {
    type BCode = str;

    fn run(&mut self, code: impl AsRef<Self::BCode>) -> Result<()> {
        let code = code.as_ref();
        let res = self.runtime.execute_script("[odra]", code).map_err(anyhow_to_eyre)?;
        let res: &Value = res.borrow();

        println!("script result: {}", res.to_rust_string_lossy(&mut self.runtime.handle_scope()));

        Ok(())
    }
}

fn anyhow_to_eyre(err: AnyError) -> Report {
    let js_error: JsError = err.downcast().unwrap();
    Report::new(js_error)
}