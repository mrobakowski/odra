#![feature(let_else)]

extern crate proc_macro;
use proc_macro::TokenStream as TokenStream1;
use proc_macro2::{Span, TokenStream};
use proc_macro_crate::{crate_name, FoundCrate};
use quote::quote;
use syn::{parse_macro_input, FnArg, Ident, ItemFn, ReturnType};

#[proc_macro_attribute]
pub fn odra_word(args: TokenStream1, body: TokenStream1) -> TokenStream1 {
    let res = odra_parse_internal(args, body, false);
    eprintln!("{res}");
    res
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

    let (args, arg_extraction): (Vec<_>, Vec<_>) = func
        .sig
        .inputs
        .iter()
        .enumerate()
        .map(|(num, arg)| {
            let FnArg::Typed(arg) = arg else { panic!("self arguments not supported") };
            let typ = &arg.ty;

            let arg = Ident::new(&format!("arg_{num}"), Span::call_site());
            let extraction = quote! {
                // SAFETY: ¯\_(ツ)_/¯
                let #arg = {
                    use #odra::spec_get_vm::*;
                    // hack for autoref specialization
                    // see https://github.com/dtolnay/case-studies/blob/master/autoref-specialization/README.md
                    // and odra/crates/core/src/spec_get_vm.rs
                    let marker: std::marker::PhantomData<#typ> = Default::default();
                    let res = (&marker).get_access_token().get(vm);

                    res
                };
            };

            (arg, extraction)
        })
        .unzip();

    let exec_impl = if is_macro {
        quote!(self.real_exec(vm))
    } else {
        quote!(vm.append_thunk(move |vm1| self.real_exec(vm1)))
    };

    let push_onto_stack = match func.sig.output {
        ReturnType::Default => quote!(),
        _ => quote!(vm.push_onto_stack(res);),
    };

    let expanded = quote! {
        #func
        #[allow(non_camel_case_types)]
        const _: () = {
            #[derive(Debug)]
            struct #struct_name;

            impl #odra::Word for #struct_name {
                fn exec(&'static self, vm: &mut Vm) {
                    #exec_impl
                }

                fn real_exec(&self, vm: &mut #odra::Vm) {
                    #(#arg_extraction)*
                    let res = #name(#(#args),*);

                    #push_onto_stack
                }

                fn name(&self) -> &str {
                    stringify!(#name)
                }

                fn is_macro(&self) -> bool { #is_macro }

                fn stack_effect(&self) -> #odra::StackEffect {
                    #stack_effect
                }

                fn register(&self, vm: &mut #odra::Vm) -> Result<(), #color_eyre::Report> {
                    vm.register(word)
                }
            };

            #[linkme::distributed_slice(#odra::words::WORDS)]
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
            quote!(<#ty as #odra::AsOdraType>::odra_type())
        }
    });

    let outs = match output {
        ReturnType::Default => quote!(vec![]),
        ReturnType::Type(_, ty) => {
            quote!(<#ty as #odra::AsOdraType>::odra_type().into_iter().collect())
        }
    };

    quote!(#odra::StackEffect::Static {
        inputs: std::array::IntoIter::new([#(#args),*]).filter_map(|x| x).collect(),
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
