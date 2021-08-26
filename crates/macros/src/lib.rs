extern crate proc_macro;
use proc_macro::TokenStream;
use quote::{quote, ToTokens};
use syn::{parse_macro_input, Ident, ItemFn};

#[proc_macro_attribute]
pub fn odra_word(args: TokenStream, body: TokenStream) -> TokenStream {
    odra_parse_internal(args, body, false)
}

#[proc_macro_attribute]
pub fn odra_macro(args: TokenStream, body: TokenStream) -> TokenStream {
    odra_parse_internal(args, body, true)
}

fn odra_parse_internal(_args: TokenStream, body: TokenStream, is_macro: bool) -> TokenStream {
    let func = parse_macro_input!(body as ItemFn);
    let name = func.sig.ident.clone();
    let struct_name = Ident::new(&format!("Word_{}", name), name.span());
    let args = func
        .sig
        .inputs
        .iter()
        .map(|elem| elem.to_token_stream().to_string());
    let outs = func.sig.output.to_token_stream().to_string();

    let stack_effect = if is_macro {
        quote!(crate::StackEffect::Dynamic)
    } else {
        quote!(crate::StackEffect::Static {
            inputs: vec![#(#args.into()),*],
            outputs: vec![#outs.into()]
        })
    };

    let expanded = quote! {
        #func
        #[allow(non_camel_case_types)]
        const _: () = {
            #[derive(Debug)]
            struct #struct_name;

            impl crate::Word for #struct_name {
                fn exec(&self, compiler: &mut dyn crate::Compiler) {
                    todo!();
                }

                fn name(&self) -> &str {
                    stringify!(#name)
                }

                fn is_macro(&self) -> bool { #is_macro }

                fn stack_effect(&self) -> crate::StackEffect {
                    #stack_effect
                }
            };

            #[linkme::distributed_slice(WORDS)]
            static word: &'static dyn Word = &#struct_name;
        };
    };

    TokenStream::from(expanded)
}
