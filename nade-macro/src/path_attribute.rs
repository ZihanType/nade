use syn::{meta::ParseNestedMeta, parse_quote, Attribute, Path};

use crate::maybe_start_with_dollar::MaybeStartWithDollar;

#[derive(Default)]
struct PathAttrOptions {
    macro_v: Option<MaybeStartWithDollar<Path>>,
    nade_helper: Option<MaybeStartWithDollar<Path>>,
}

impl PathAttrOptions {
    fn parse_meta(&mut self, meta: ParseNestedMeta) -> syn::Result<()> {
        macro_rules! parse_path {
            ($argument:tt) => {
                if meta.path.is_ident(stringify!($argument)) {
                    if self.$argument.is_some() {
                        return Err(meta.error(concat!(
                            "duplicate `",
                            stringify!($argument),
                            "` argument"
                        )));
                    }
                    self.$argument = Some(meta.value()?.parse()?);
                    return Ok(());
                }
            };
        }

        parse_path!(macro_v);
        parse_path!(nade_helper);

        Err(meta.error("the argument must be one of: `macro_v`, `nade_helper`"))
    }

    fn parse_attr(&mut self, attr: &Attribute) -> syn::Result<()> {
        attr.parse_nested_meta(|meta| self.parse_meta(meta))
    }
}

pub(crate) struct PathAttr {
    pub(crate) macro_v: MaybeStartWithDollar<Path>,
    pub(crate) nade_helper: MaybeStartWithDollar<Path>,
}

impl PathAttr {
    pub(crate) fn parse_attrs(attrs: &mut Vec<Attribute>) -> syn::Result<Self> {
        let mut options = PathAttrOptions::default();
        let mut errors = Vec::new();

        attrs.retain(|attr| {
            if !attr.path().is_ident("nade_path") {
                return true;
            }

            if let Err(e) = options.parse_attr(attr) {
                errors.push(e);
            }

            false
        });

        if let Some(e) = errors.into_iter().reduce(|mut a, b| {
            a.combine(b);
            a
        }) {
            return Err(e);
        }

        let PathAttrOptions {
            macro_v,
            nade_helper,
        } = options;

        Ok(PathAttr {
            macro_v: macro_v.unwrap_or_else(|| parse_quote!(::nade::__internal)),
            nade_helper: nade_helper.unwrap_or_else(|| parse_quote!($crate)),
        })
    }
}
