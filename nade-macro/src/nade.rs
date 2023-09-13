use proc_macro2::TokenStream;
use quote::quote;
use syn::{
    parse_quote, spanned::Spanned, AttrStyle, Attribute, Expr, ExprLit, File, FnArg, Ident, Item,
    ItemFn, Lit, LitStr, Meta, MetaList, MetaNameValue, PatType, Path, PathSegment,
};

use crate::{
    maybe_starts_with_dollar::{MaybeStartsWithDollar, StartsWithDollar},
    parameter::Parameter,
    parameter_doc::ParameterDoc,
    path_attribute::{PathAttribute, SimplePathAttribute},
};

pub(crate) fn generate(
    module_path: Option<StartsWithDollar<Path>>,
    fun: &mut ItemFn,
) -> syn::Result<TokenStream> {
    let SimplePathAttribute {
        macro_v: macro_v_path,
        nade_helper: nade_helper_path,
    } = PathAttribute::try_from(&mut fun.attrs)?.simplify();

    let (parameters, parameter_docs) = extract_parameters_and_docs(fun)?;

    let name = &fun.sig.ident;

    let fn_docs = generate_fn_docs(
        &fun.attrs,
        name,
        module_path
            .as_ref()
            .map(|path| simple_path_to_string(&path.inner)),
    );

    let vis = &fun.vis;

    let parameter_docs = generate_parameter_docs(parameter_docs);

    let module_path = module_path.map(|path| quote!(#path::));

    let expand = quote! {
        #[allow(clippy::too_many_arguments)]
        #fun

        #[#macro_v_path::macro_v(#vis)]
        #fn_docs
        #parameter_docs
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
    fun: &mut ItemFn,
) -> syn::Result<(Vec<Parameter>, Vec<ParameterDoc>)> {
    let mut parameters = Vec::new();
    let mut parameter_docs = Vec::new();

    for arg in fun.sig.inputs.iter_mut() {
        match arg {
            FnArg::Receiver(r) => {
                return Err(syn::Error::new(r.span(), "`self` is not support"));
            }
            FnArg::Typed(PatType { attrs, pat, ty, .. }) => {
                let mut nade_attrs = drain_filter(attrs, |attr| attr.path().is_ident("nade"));

                if nade_attrs.len() > 1 {
                    const MSG: &str = "`#[nade(..)]` can only be used once per parameter";

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
                        check_unexpected_expr_type(expr.inner())?;
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

                parameters.push(Parameter {
                    pattern: *pat.clone(),
                    default,
                });
            }
        }
    }

    Ok((parameters, parameter_docs))
}

fn generate_fn_docs<'a>(
    attrs: &'a [Attribute],
    name: &'a Ident,
    module_path: Option<String>,
) -> TokenStream {
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

    let link_doc = if let Some(module_path) = module_path {
        format!(
            "Wrapper macro for function [`{}`]({}::{}()).",
            name, module_path, name
        )
    } else {
        format!("Wrapper macro for function [`{}`]({}()).", name, name)
    };

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

    let docs = docs.into_iter().map(generate_one_parameter_doc);
    quote! {
        #[doc = "# Parameters"]
        #(#docs)*
    }
}

fn generate_one_parameter_doc(parameter_doc: ParameterDoc) -> TokenStream {
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

    let parameter_define = if let Some(default) = default {
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

fn extract_parameter_default(attr: Attribute) -> syn::Result<MaybeStartsWithDollar<Expr>> {
    let meta = attr.meta;

    match meta {
        Meta::Path(_) => Ok(MaybeStartsWithDollar::Normal(parse_quote!(
            ::core::default::Default::default()
        ))),
        Meta::List(MetaList { tokens, .. }) => syn::parse2(tokens),
        Meta::NameValue(a) => Err(syn::Error::new(
            a.span(),
            "The `#[nade]` attribute does not support `#[nade = ..]`",
        )),
    }
}

fn check_unexpected_expr_type(expr: &Expr) -> syn::Result<()> {
    if let Expr::Assign(_) = expr {
        Err(syn::Error::new(
            expr.span(),
            "assignment expression is not supported \
                because it is not possible to distinguish \
                whether it is a named general expression \
                or a non-named assignment expression.",
        ))
    } else {
        Ok(())
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

fn simple_path_to_string(
    Path {
        leading_colon,
        segments,
    }: &Path,
) -> String {
    if segments.is_empty() {
        return String::new();
    }

    let mut buf = String::with_capacity(segments.len() * 2);

    if leading_colon.is_some() {
        buf.push_str("::");
    }

    let mut first = true;

    segments.iter().for_each(|PathSegment { ident, .. }| {
        if first {
            first = false;
        } else {
            buf.push_str("::");
        }

        buf.push_str(&ident.to_string());
    });

    buf
}
