use proc_macro2::TokenStream;
use quote::{quote, ToTokens};
use syn::{
    parse_quote, spanned::Spanned, AttrStyle, Attribute, Expr, ExprLit, File, FnArg, Ident, Item,
    ItemFn, Lit, LitStr, Meta, MetaNameValue, Path,
};

use crate::{
    maybe_starts_with_dollar::{MaybeStartsWithDollar, Normal, StartsWithDollar},
    parameter::Parameter,
    parameter_doc::ParameterDoc,
};

pub(crate) fn generate(
    module_path: Option<StartsWithDollar<Path>>,
    fun: &mut ItemFn,
) -> syn::Result<TokenStream> {
    let (parameters, parameter_docs) = get_parameters_and_docs(fun)?;

    let name = &fun.sig.ident;

    let fn_path = module_path.map(|path| quote!(#path::));

    let vis = &fun.vis;

    let fn_docs = get_fn_docs(&fun.attrs, name);
    let parameter_docs = get_parameter_docs(parameter_docs);

    let expand = quote! {
        #[allow(clippy::too_many_arguments)]
        #fun

        #[::nade::__internal::macro_v(#vis)]
        #fn_docs
        #parameter_docs
        macro_rules! #name {
            ($($arguments:tt)*) => {
                $crate::nade_helper!(
                    ($($arguments)*)
                    (#(#parameters),*)
                    (#fn_path #name)
                )
            }
        }
    };

    Ok(expand)
}

fn get_parameters_and_docs(fun: &mut ItemFn) -> syn::Result<(Vec<Parameter>, Vec<ParameterDoc>)> {
    let mut parameters = Vec::new();
    let mut parameter_docs = Vec::new();

    for arg in fun.sig.inputs.iter_mut() {
        match arg {
            FnArg::Receiver(r) => {
                return Err(syn::Error::new(r.span(), "`self` is not support"));
            }
            FnArg::Typed(pat_type) => {
                let mut nade_attrs =
                    drain_filter(&mut pat_type.attrs, |attr| attr.path().is_ident("nade"));

                if nade_attrs.len() > 1 {
                    return Err(syn::Error::new(
                        pat_type.span(),
                        "`#[nade(..)]` can only be used once per parameter",
                    ));
                }

                let doc_attrs =
                    drain_filter(&mut pat_type.attrs, |attr| attr.path().is_ident("doc"));

                let pat = *pat_type.pat.clone();
                let doc_pat = pat.clone();

                let default = match nade_attrs.pop() {
                    Some(nade_attr) => {
                        let expr = get_parameter_default(nade_attr)?;
                        check_unexpected_expr_type(expr.inner())?;
                        Some(expr)
                    }
                    None => None,
                };
                let doc_default = default.as_ref().map(|d| d.inner().clone());

                parameters.push(Parameter {
                    pattern: pat,
                    default,
                });

                let ty = *pat_type.ty.clone();

                let docs = doc_attrs
                    .into_iter()
                    .filter_map(|attr| {
                        if !matches!(attr.style, AttrStyle::Outer) {
                            return None;
                        }

                        if let Meta::NameValue(MetaNameValue {
                            value:
                                Expr::Lit(ExprLit {
                                    lit: Lit::Str(s), ..
                                }),
                            ..
                        }) = attr.meta
                        {
                            return Some(s);
                        }

                        None
                    })
                    .collect::<Vec<_>>();

                parameter_docs.push(ParameterDoc {
                    pattern: doc_pat,
                    ty,
                    docs,
                    default: doc_default,
                });
            }
        }
    }

    Ok((parameters, parameter_docs))
}

fn get_fn_docs<'a>(attrs: &'a [Attribute], name: &'a Ident) -> TokenStream {
    let mut has_doc_comment = false;

    let fn_docs = attrs
        .iter()
        .filter(|attr| matches!(attr.style, AttrStyle::Outer) && attr.path().is_ident("doc"))
        .inspect(|attr| {
            if !has_doc_comment {
                if let Meta::NameValue(_) = attr.meta {
                    has_doc_comment = true;
                }
            }
        })
        .collect::<Vec<_>>();

    let blank_line = if has_doc_comment {
        quote! {
            #[doc = ""]
        }
    } else {
        quote! {}
    };

    let link_to_fn = LitStr::new(
        &format!("Wrapper macro for function [`{}()`].", quote!(#name)),
        name.span(),
    );

    quote! {
        #(#fn_docs)*
        #blank_line
        #[doc = #link_to_fn]
    }
}

fn get_parameter_docs(docs: Vec<ParameterDoc>) -> TokenStream {
    if docs.is_empty() {
        return quote! {};
    }

    let docs = docs.into_iter().map(parameter_to_doc);
    quote! {
        #[doc = "# Parameters"]
        #(#docs)*
    }
}

fn parameter_to_doc(parameter_doc: ParameterDoc) -> TokenStream {
    let ParameterDoc {
        pattern,
        ty,
        docs,
        default,
    } = parameter_doc;

    let type_item: Item = parse_quote! {
        type SomeType = #ty;
    };

    let file = File {
        shebang: None,
        attrs: vec![],
        items: vec![type_item],
    };

    let pretty_ty = prettyplease::unparse(&file);
    let pretty_ty = &pretty_ty[16..&pretty_ty.len() - 2];

    let type_doc = if let Some(default) = default {
        let expr_item: Item = parse_quote! {
            fn a() {
                let _ = #default;
            }
        };

        let file = File {
            shebang: None,
            attrs: vec![],
            items: vec![expr_item],
        };

        let pretty_expr = prettyplease::unparse(&file);
        let pretty_expr = &pretty_expr[21..&pretty_expr.len() - 4];

        format!(
            "- **{}**: [`{}`] = {}",
            quote!(#pattern),
            pretty_ty,
            pretty_expr
        )
    } else {
        format!("- **{}**: [`{}`]", quote!(#pattern), pretty_ty)
    };

    let type_doc = LitStr::new(&type_doc, pattern.span());

    let para_docs = docs.into_iter().enumerate().map(|(idx, doc)| {
        let doc_str = doc.value();
        let doc_str = if idx == 0 {
            format!("    - {doc_str}")
        } else {
            format!("      {doc_str}")
        };

        LitStr::new(&doc_str, doc.span())
    });

    quote! {
        #[doc = #type_doc]
        #(#[doc = #para_docs])*
    }
}

fn get_parameter_default(attr: Attribute) -> syn::Result<MaybeStartsWithDollar<Expr>> {
    if let Meta::Path(_) = attr.meta {
        return Ok(MaybeStartsWithDollar::Normal(Normal {
            inner: syn::parse2(quote!(::core::default::Default::default()))?,
        }));
    }

    let tokens: proc_macro::TokenStream = attr.meta.require_list()?.tokens.to_token_stream().into();

    MaybeStartsWithDollar::try_from(tokens)
}

fn check_unexpected_expr_type(expr: &Expr) -> syn::Result<()> {
    macro_rules! err {
        ($v:tt) => {
            Err(syn::Error::new(
                expr.span(),
                concat!($v, " is not supported in `#[nade(..)]`"),
            ))
        };
    }

    match &expr {
        Expr::Assign(_) => err!("assignment"),
        Expr::Await(_) => err!("`fut.await`"),
        Expr::Break(_) => err!("`break`"),
        Expr::Continue(_) => err!("`continue`"),
        Expr::Let(_) => err!("`let` guard"),
        Expr::Return(_) => err!("`return`"),
        Expr::Try(_) => err!("`expr?`"),
        Expr::Yield(_) => err!("`yield expr`"),
        _ => Ok(()),
    }
}

// implemented manually because Vec::drain_filter is nightly only
// follows std recommended parallel
fn drain_filter<T, F>(vec: &mut Vec<T>, mut predicate: F) -> Vec<T>
where
    F: FnMut(&mut T) -> bool,
{
    let mut ret = Vec::new();
    let mut i = 0;
    while i < vec.len() {
        if predicate(&mut vec[i]) {
            ret.push(vec.remove(i));
        } else {
            i += 1;
        }
    }
    ret
}
