use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::{
    Expr, Ident, ItemFn, LitStr, Path, ReturnType, Token,
    parse::{Parse, ParseStream},
    parse_macro_input,
    spanned::Spanned,
};

struct ContextArgs {
    format_str: LitStr,
    format_args: Vec<Expr>,
    anyhow_path: Path,
}

impl Parse for ContextArgs {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let format_str: LitStr = input.parse()?;
        let mut format_args = Vec::new();
        let mut anyhow_path: Path = syn::parse_quote!(::anyhow);

        while !input.is_empty() {
            input.parse::<Token![,]>()?;
            if input.is_empty() {
                break;
            }

            let is_anyhow_kwarg = {
                let fork = input.fork();
                matches!(
                    fork.parse::<Ident>(),
                    Ok(ident) if ident == "anyhow" && fork.peek(Token![=])
                )
            };

            if is_anyhow_kwarg {
                input.parse::<Ident>()?;
                input.parse::<Token![=]>()?;
                anyhow_path = input.parse()?;
            } else {
                format_args.push(input.parse()?);
            }
        }

        Ok(ContextArgs {
            format_str,
            format_args,
            anyhow_path,
        })
    }
}

#[proc_macro_attribute]
pub fn context(attr: TokenStream, item: TokenStream) -> TokenStream {
    let args = parse_macro_input!(attr as ContextArgs);
    let func = parse_macro_input!(item as ItemFn);
    expand(args, func).into()
}

fn expand(args: ContextArgs, func: ItemFn) -> TokenStream2 {
    let ItemFn {
        attrs,
        vis,
        sig,
        block,
    } = func;

    let output_ty = match &sig.output {
        ReturnType::Default => {
            return syn::Error::new(
                sig.span(),
                "#[context] requires an explicit return type, e.g. `-> anyhow::Result<T>`",
            )
            .to_compile_error();
        }
        ReturnType::Type(_, ty) => ty.clone(),
    };

    let ContextArgs {
        format_str,
        format_args,
        anyhow_path,
    } = args;

    let is_async = sig.asyncness.is_some();
    let is_unsafe = sig.unsafety.is_some();

    let body: TokenStream2 = if is_unsafe {
        quote! { { unsafe #block } }
    } else {
        quote! { #block }
    };

    let invoke_expr = if is_async {
        quote! { (async #body).await }
    } else {
        quote! { (|| -> #output_ty #body)() }
    };

    let format_args_tokens = if format_args.is_empty() {
        quote! {}
    } else {
        quote! { , #(#format_args),* }
    };

    quote! {
        #(#attrs)*
        #vis #sig {
            #anyhow_path::Context::with_context(
                #invoke_expr,
                || format!(#format_str #format_args_tokens),
            )
        }
    }
}
