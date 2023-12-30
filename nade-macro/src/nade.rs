use proc_macro2::TokenStream;
use quote::quote;
use syn::{
    parse_quote, punctuated::Punctuated, spanned::Spanned, AttrStyle, Attribute, Expr, ExprLit,
    File, FnArg, Ident, Item, ItemFn, Lit, LitStr, Meta, MetaList, MetaNameValue, Pat, PatType,
    Path, ReturnType, Token, Type,
};

use crate::{
    maybe_start_with_dollar::{MaybeStartWithDollar, StartWithDollar},
    parameter::Parameter,
    parameter_doc::ParameterDoc,
    path_attribute::PathAttr,
};

pub(crate) fn generate(
    module_path: Option<StartWithDollar<Path>>,
    fun: &mut ItemFn,
) -> syn::Result<TokenStream> {
    let PathAttr {
        macro_v: macro_v_path,
        nade_helper: nade_helper_path,
    } = PathAttr::parse_attrs(&mut fun.attrs)?;

    let (parameters, parameter_docs) = extract_parameters_and_docs(&mut fun.sig.inputs)?;

    let name = &fun.sig.ident;
    let vis = &fun.vis;

    let macro_docs = generate_macro_docs(&fun.attrs, name);
    let parameter_docs = generate_parameter_docs(parameter_docs);
    let return_doc = generate_return_doc(&fun.sig.output);

    let module_path = module_path.map(|path| quote!(#path::));

    let expand = quote! {
        #[allow(clippy::too_many_arguments)]
        #fun

        #[#macro_v_path::macro_v(#vis)]
        #macro_docs
        #parameter_docs
        #return_doc
        macro_rules! #name {
            ($($arguments:tt)*) => {
                #nade_helper_path::nade_helper!(
                    ($($arguments)*)
                    (#(#parameters,)*)
                    (#module_path #name)
                )
            }
        }
    };

    Ok(expand)
}

fn extract_parameters_and_docs(
    inputs: &mut Punctuated<FnArg, Token![,]>,
) -> syn::Result<(Vec<Parameter>, Vec<ParameterDoc>)> {
    let mut parameters = Vec::new();
    let mut parameter_docs = Vec::new();

    for arg in inputs.iter_mut() {
        match arg {
            FnArg::Receiver(r) => {
                return Err(syn::Error::new(r.span(), "`self` is not support"));
            }
            FnArg::Typed(PatType {
                attrs,
                pat,
                colon_token,
                ty,
            }) => {
                let mut nade_attrs = drain_filter(attrs, |attr| attr.path().is_ident("nade"));

                if nade_attrs.len() > 1 {
                    const MSG: &str =
                        "the `#[nade(..)]` attribute can only be used once per parameter";

                    if let Some(e) = nade_attrs
                        .iter()
                        .skip(1)
                        .map(|attr| syn::Error::new(attr.span(), MSG))
                        .reduce(|mut a, b| {
                            a.combine(b);
                            a
                        })
                    {
                        return Err(e);
                    }
                }

                let doc_attrs = drain_filter(attrs, |attr| attr.path().is_ident("doc"));

                let default = match nade_attrs.pop() {
                    Some(nade_attr) => {
                        let expr = extract_parameter_default(nade_attr)?;
                        Some(expr)
                    }
                    None => None,
                };

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
                    pattern: *pat.clone(),
                    ty: *ty.clone(),
                    docs,
                    default: default.as_ref().map(|d| d.inner().clone()),
                });

                parameters.push(Parameter::new(
                    *pat.clone(),
                    *colon_token,
                    *ty.clone(),
                    default,
                ));
            }
        }
    }

    Ok((parameters, parameter_docs))
}

fn generate_macro_docs<'a>(attrs: &'a [Attribute], name: &'a Ident) -> TokenStream {
    let mut has_doc_comment = false;

    let fn_docs = attrs
        .iter()
        .filter(|attr| matches!(attr.style, AttrStyle::Outer) && attr.path().is_ident("doc"))
        .inspect(|attr| {
            if !has_doc_comment && matches!(attr.meta, Meta::NameValue(_)) {
                has_doc_comment = true;
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

    let link_doc = format!("Wrapper macro for function [`{}`]({}()).", name, name);

    let link_to_fn = LitStr::new(&link_doc, name.span());

    quote! {
        #(#fn_docs)*
        #blank_line
        #[doc = #link_to_fn]
    }
}

fn generate_parameter_docs(docs: Vec<ParameterDoc>) -> TokenStream {
    if docs.is_empty() {
        return quote! {};
    }

    let docs = docs.into_iter().map(generate_single_parameter_doc);
    quote! {
        #[doc = "## Parameters"]
        #(#docs)*
    }
}

fn generate_single_parameter_doc(parameter_doc: ParameterDoc) -> TokenStream {
    let ParameterDoc {
        pattern,
        ty,
        docs,
        default,
    } = parameter_doc;

    let pretty_pattern = generate_pretty_pat(&pattern);
    let pretty_ty = generate_pretty_ty(&ty);

    let pretty_default = default
        .map(|expr| format!(" = {}", generate_pretty_expr(&expr)))
        .unwrap_or_default();

    let parameter_define = format!(
        "- **{}** : [`{}`]{}",
        pretty_pattern, pretty_ty, pretty_default
    );

    let parameter_define = LitStr::new(&parameter_define, pattern.span());

    let parameter_docs = docs.into_iter().enumerate().map(|(idx, doc)| {
        let doc_str = doc.value();
        let doc_str = if idx == 0 {
            format!("    - {doc_str}")
        } else {
            format!("      {doc_str}")
        };

        LitStr::new(&doc_str, doc.span())
    });

    quote! {
        #[doc = #parameter_define]
        #(#[doc = #parameter_docs])*
    }
}

fn generate_return_doc(output: &ReturnType) -> TokenStream {
    let ty_doc = match output {
        ReturnType::Default => "[`()`](unit)".to_string(),
        ReturnType::Type(_, ty) => {
            format!("[`{}`]", generate_pretty_ty(ty))
        }
    };

    quote! {
        #[doc = "## Return"]
        #[doc = #ty_doc]
    }
}

fn extract_parameter_default(attr: Attribute) -> syn::Result<MaybeStartWithDollar<Expr>> {
    let default = match attr.meta {
        Meta::Path(_) => Ok(MaybeStartWithDollar::Normal(parse_quote!(
            ::core::default::Default::default()
        ))),
        Meta::List(MetaList { tokens, .. }) => syn::parse2(tokens),
        Meta::NameValue(a) => Err(syn::Error::new(
            a.span(),
            "the `#[nade]` attribute does not support `#[nade = ..]`",
        )),
    }?;

    if let Expr::Assign(_) = default.inner() {
        return Err(syn::Error::new(
            default.span(),
            "assignment expression is not supported \
                because it is not possible to distinguish \
                whether it is a named general expression \
                or a non-named assignment expression.",
        ));
    }

    Ok(default)
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

fn item_to_file(item: Item) -> File {
    File {
        shebang: None,
        attrs: vec![],
        items: vec![item],
    }
}

fn generate_pretty_pat(pat: &Pat) -> String {
    let pat_item: Item = parse_quote! {
        fn a() {
            let #pat = todo!();
        }
    };

    let file = item_to_file(pat_item);

    let pretty_pat = prettyplease::unparse(&file);
    let pretty_pat = &pretty_pat[17..&pretty_pat.len() - 14];

    pretty_pat.to_string()
}

fn generate_pretty_ty(ty: &Type) -> String {
    let type_item: Item = parse_quote! {
        type SomeType = #ty;
    };

    let file = item_to_file(type_item);

    let pretty_ty = prettyplease::unparse(&file);
    let pretty_ty = &pretty_ty[16..&pretty_ty.len() - 2];

    pretty_ty.to_string()
}

fn generate_pretty_expr(expr: &Expr) -> String {
    let expr_item: Item = parse_quote! {
        fn a() {
            let _ = #expr;
        }
    };

    let file = item_to_file(expr_item);

    let pretty_expr = prettyplease::unparse(&file);
    let pretty_expr = &pretty_expr[21..&pretty_expr.len() - 4];

    pretty_expr.to_string()
}
