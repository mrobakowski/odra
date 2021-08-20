use color_eyre::{Report, Result};
use core::fmt::Debug;
use std::fmt::Write;
use v8::{Context, Global, HandleScope, Isolate, Local, OwnedIsolate, Script, TryCatch};

pub(crate) trait Vm {
    type BCode: Debug + ?Sized;
    fn run(&mut self, code: impl AsRef<Self::BCode>) -> Result<()>;
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
        let platform =
            v8::new_default_platform(0 /* 0 gets number of cores */, false).make_shared();
        v8::V8::initialize_platform(platform);
        v8::V8::initialize();

        let mut isolate = Isolate::new(v8::CreateParams::default());
        let context = {
            let mut handle_scope = HandleScope::new(&mut isolate);
            let local_context = Context::new(&mut handle_scope);
            Global::new(&mut handle_scope, local_context)
        };

        V8Vm { isolate, context }
    }
}

impl Vm for V8Vm {
    type BCode = str;

    fn run(&mut self, code: impl AsRef<Self::BCode>) -> Result<()> {
        let code = code.as_ref();
        let V8Vm { isolate, context } = self;
        let scope = &mut HandleScope::with_context(isolate, &*context);
        let scope = &mut TryCatch::new(scope);

        let code = v8::String::new(scope, code).expect("unreasonably long code");
        let script = Script::compile(scope, code, None).ok_or_else(|| exception_report(scope))?;
        let res = script.run(scope).ok_or_else(|| exception_report(scope))?;

        println!("script result: {}", res.to_rust_string_lossy(scope));

        Ok(())
    }
}

fn exception_report(try_catch: &mut TryCatch<HandleScope>) -> Report {
    assert!(try_catch.has_caught());

    let mut buffer = String::with_capacity(100);

    let exception = try_catch.exception().unwrap();
    let exception_string = exception
        .to_string(try_catch)
        .unwrap()
        .to_rust_string_lossy(try_catch);
    let message = if let Some(message) = try_catch.message() {
        message
    } else {
        writeln!(&mut buffer, "{}", exception_string).unwrap();
        return Report::msg(buffer);
    };

    // Print (filename):(line number): (message).
    let filename = message.get_script_resource_name(try_catch).map_or_else(
        || "(unknown)".into(),
        |s| {
            s.to_string(try_catch)
                .unwrap()
                .to_rust_string_lossy(try_catch)
        },
    );
    let line_number = message.get_line_number(try_catch).unwrap_or_default();

    writeln!(
        &mut buffer,
        "{}:{}: {}",
        filename, line_number, exception_string
    )
    .unwrap();

    // Print line of source code.
    let source_line = message
        .get_source_line(try_catch)
        .map(|s| {
            s.to_string(try_catch)
                .unwrap()
                .to_rust_string_lossy(try_catch)
        })
        .unwrap();
    writeln!(&mut buffer, "{}", source_line).unwrap();

    // Print wavy underline (GetUnderline is deprecated).
    let start_column = message.get_start_column();
    let end_column = message.get_end_column();

    for _ in 0..start_column {
        write!(buffer, " ").unwrap();
    }

    for _ in start_column..end_column {
        write!(buffer, "^").unwrap();
    }

    writeln!(&mut buffer).unwrap();

    // Print stack trace
    let stack_trace = if let Some(stack_trace) = try_catch.stack_trace() {
        stack_trace
    } else {
        return Report::msg(buffer);
    };
    let stack_trace = unsafe { Local::<v8::String>::cast(stack_trace) };
    let stack_trace = stack_trace
        .to_string(try_catch)
        .map(|s| s.to_rust_string_lossy(try_catch));

    if let Some(stack_trace) = stack_trace {
        writeln!(&mut buffer, "{}", stack_trace).unwrap();
    }

    Report::msg(buffer)
}
