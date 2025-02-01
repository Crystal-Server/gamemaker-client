use proc_macro::TokenStream;
use proc_macro2::{Ident, Span};
use quote::quote;
use syn::{
    parse::{Parse, ParseStream},
    parse_macro_input, GenericParam, Generics, ItemFn, LitStr, PatType, Signature, Visibility,
};

#[allow(dead_code)]
enum Args {
    Default,
    Custom(LitStr),
}

impl Parse for Args {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        match input.parse() {
            Ok(i) => Ok(Args::Custom(i)),
            Err(_) => Ok(Args::Default),
        }
    }
}

/// Exposes your function to Gamemaker FFI.
///
/// All argument types must implement `gm_utils::func::GmArg` and the return type must implement `gm_utils::func::GmReturn`.
///
/// You may optionally specify the external name of the function.
#[proc_macro_attribute]
pub fn gm_func(attr: TokenStream, input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as ItemFn);
    let attr = parse_macro_input!(attr as Option<LitStr>);

    let ItemFn { vis, sig, .. } = input.clone();

    let Signature {
        asyncness,
        mut ident,
        generics,
        inputs,
        variadic,
        output,
        ..
    } = sig;

    match vis {
        Visibility::Public(_) => (),
        _ => return quote!(compile_error!("Function must be `pub`.")).into(),
    }

    if asyncness.is_some() {
        return quote!(compile_error!("Function may not be `async`.")).into();
    }

    let Generics {
        params,
        where_clause,
        ..
    } = generics;

    for p in &params {
        match p {
            GenericParam::Lifetime(_) => (),
            _ => {
                return quote!(compile_error!(
                    "Function may only be generic over lifetimes."
                ))
                .into()
            }
        }
    }

    let where_clause = where_clause.into_iter();
    let params = params.into_iter();

    if variadic.is_some() {
        return quote!(compile_error!("Function may not be variadic.")).into();
    }

    let output = match output {
        syn::ReturnType::Default => quote!(<() as ::gm_utils::func::GmReturn>::Return),
        syn::ReturnType::Type(_, t) => quote!(<#t as ::gm_utils::func::GmReturn>::Return),
    };

    let attr = match attr {
        Some(str) => quote!(#[export_name = #str]),
        None => quote!(#[no_mangle]),
    };

    let args = inputs.iter().enumerate().map(|(i, a)| match a {
        syn::FnArg::Receiver(_) => {
            quote!(compile_error!("Function may not receive `self` arguments."))
        }
        syn::FnArg::Typed(a) => {
            let PatType { attrs, ty, .. } = a;
            let name = Ident::new(&format!("arg_{i}"), Span::call_site());
            quote!(#(#attrs)* #name : <#ty as ::gm_utils::func::GmArg>::Arg)
        }
    });
    let names = (0..inputs.len()).map(|i| {
        let name = Ident::new(&format!("arg_{i}"), Span::call_site());
        quote!(#name)
    });

    ident.set_span(Span::call_site());

    let mod_name = Ident::new(&format!("__mod_gm_export_{}", ident), Span::call_site());

    quote! {
        #input

        mod #mod_name {
            use super::*;
            #attr
            pub unsafe fn #ident<#(#params),*>(#(#args),*) #(#where_clause)* -> #output {
                match ::std::panic::catch_unwind(|| ::gm_utils::func::GmReturn::to_return(super::#ident(#(::gm_utils::func::GmArg::to_arg(#names)),*))) {
                    Ok(o) => o,
                    Err(_) => ::gm_utils::__private::GmDefault::default(),
                }
            }
        }
    }
    .into()
}
