extern crate proc_macro;
use proc_macro::TokenStream as TokenStream1;
use proc_macro2::TokenStream;
use quote::{quote, ToTokens};
use syn::{parse_macro_input, FnArg, Ident, ItemFn, ReturnType};

#[proc_macro_attribute]
pub fn odra_word(args: TokenStream1, body: TokenStream1) -> TokenStream1 {
    odra_parse_internal(args, body, false)
}

#[proc_macro_attribute]
pub fn odra_macro(args: TokenStream1, body: TokenStream1) -> TokenStream1 {
    odra_parse_internal(args, body, true)
}

fn odra_parse_internal(_args: TokenStream1, body: TokenStream1, is_macro: bool) -> TokenStream1 {
    let func = parse_macro_input!(body as ItemFn);
    let name = func.sig.ident.clone();
    let struct_name = Ident::new(&format!("Word_{}", name), name.span());

    let stack_effect = if is_macro {
        quote!(crate::StackEffect::Dynamic)
    } else {
        make_stack_effect(func.sig.inputs.iter(), &func.sig.output)
    };

    // TODO: more resilient `crate`
    let expanded = quote! {
        #func
        #[allow(non_camel_case_types)]
        const _: () = {
            #[derive(Debug)]
            struct #struct_name;

            impl crate::Word for #struct_name {
                fn exec(&self, vm: &mut crate::Vm) {
                    todo!();
                }

                fn name(&self) -> &str {
                    stringify!(#name)
                }

                fn is_macro(&self) -> bool { #is_macro }

                fn stack_effect(&self) -> crate::StackEffect {
                    #stack_effect
                }

                fn register(&self, vm: &mut crate::Vm) {
                    vm.register(word, #name)
                }
            };

            #[linkme::distributed_slice(WORDS)]
            static word: &'static dyn Word = &#struct_name;
        };
    };

    TokenStream1::from(expanded)
}

fn make_stack_effect<'a>(
    inputs: impl Iterator<Item = &'a FnArg>,
    output: &ReturnType,
) -> TokenStream {
    // TODO: clean this up
    let args = inputs.map(|elem| elem.to_token_stream().to_string());
    let outs = output.to_token_stream().to_string();

    quote!(crate::StackEffect::Static {
        inputs: vec![#(#args.into()),*],
        outputs: vec![#outs.into()]
    })
}
