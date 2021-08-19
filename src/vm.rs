use std::error::Error;

use v8::{Context, Global, HandleScope, OwnedIsolate, Script, TryCatch};

pub(crate) trait Vm {
    type BCode: ?Sized;
    fn run(&mut self, code: impl AsRef<Self::BCode>) -> Result<(), Box<dyn Error>>;
}

pub(crate) fn default_vm() -> impl Vm<BCode = str> {
    V8Vm::new()
}

struct V8Vm {
    isolate: OwnedIsolate,
    context: Global<Context>,
}

impl V8Vm {
    fn new() -> V8Vm {
        let platform = v8::new_default_platform(0, false).make_shared();
        v8::V8::initialize_platform(platform);
        v8::V8::initialize();

        let mut isolate = v8::Isolate::new(v8::CreateParams::default());
        let context = {
            let mut handle_scope = v8::HandleScope::new(&mut isolate);
            let local_context = v8::Context::new(&mut handle_scope);
            Global::new(&mut handle_scope, local_context)
        };

        V8Vm { isolate, context }
    }
}

impl Vm for V8Vm {
    type BCode = str;

    fn run(&mut self, code: impl AsRef<Self::BCode>) -> Result<(), Box<dyn Error>> {
        let code = code.as_ref();
        let V8Vm { isolate, context } = self;
        let scope = &mut HandleScope::with_context(isolate, &*context);
        let scope = &mut TryCatch::new(scope);

        let code = v8::String::new(scope, code).expect("unreasonably long code");

        let script = if let Some(script) = Script::compile(scope, code, None) {
            script
        } else {
            assert!(scope.has_caught());
            let exc = scope.exception().unwrap();
            return Err(exc.to_rust_string_lossy(scope).into());
        };

        let res = if let Some(res) = script.run(scope) {
            res
        } else {
            assert!(scope.has_caught());
            let exc = scope.exception().unwrap();
            return Err(exc.to_rust_string_lossy(scope).into());
        };

        println!("script result: {}", res.to_rust_string_lossy(scope));

        Ok(())
    }
}
