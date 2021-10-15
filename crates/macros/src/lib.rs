extern crate proc_macro;
use proc_macro::TokenStream as TokenStream1;
use proc_macro2::{Span, TokenStream};
use proc_macro_crate::{crate_name, FoundCrate};
use quote::quote;
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

    let odra = get_crate("odra");
    let color_eyre = get_crate("color-eyre");

    let stack_effect = if is_macro {
        quote!(crate::StackEffect::Dynamic)
    } else {
        make_stack_effect(func.sig.inputs.iter(), &func.sig.output, &odra)
    };

    let expanded = quote! {
        #func
        #[allow(non_camel_case_types)]
        const _: () = {
            #[derive(Debug)]
            struct #struct_name;

            impl #odra::Word for #struct_name {
                fn exec(&self, vm: &mut #odra::Vm) {
                    todo!();
                }

                fn name(&self) -> &str {
                    stringify!(#name)
                }

                fn is_macro(&self) -> bool { #is_macro }

                fn stack_effect(&self) -> #odra::StackEffect {
                    #stack_effect
                }

                fn register(&self, vm: &mut #odra::Vm) -> Result<(), #color_eyre::Report> {
                    vm.register(word, #name)
                }
            };

            #[linkme::distributed_slice(WORDS)]
            static word: &'static dyn #odra::Word = &#struct_name;
        };
    };

    TokenStream1::from(expanded)
}

fn make_stack_effect<'a>(
    inputs: impl Iterator<Item = &'a FnArg>,
    output: &ReturnType,
    odra: &TokenStream,
) -> TokenStream {
    let args = inputs.map(|elem| match elem {
        FnArg::Receiver(_) => panic!("unsupported self argument"),
        FnArg::Typed(pat) => {
            let ty = &pat.ty;
            quote!(<#ty as #odra::AsOdraValue>::odra_type())
        }
    });

    let outs = match output {
        ReturnType::Default => quote!(vec![]),
        ReturnType::Type(_, ty) => quote!(vec![<#ty as #odra::AsOdraValue>::odra_type()]),
    };

    quote!(#odra::StackEffect::Static {
        inputs: vec![#(#args),*],
        outputs: #outs
    })
}

fn get_crate(name: &str) -> TokenStream {
    let found_crate = crate_name(name).expect(&format!("{} is not present in `Cargo.toml`", name));

    match found_crate {
        FoundCrate::Itself => quote!(crate),
        FoundCrate::Name(name) => {
            let ident = Ident::new(&name, Span::call_site());
            quote!(#ident)
        }
    }
}
