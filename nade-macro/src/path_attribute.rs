use syn::{meta::ParseNestedMeta, parse_quote, Attribute, Path};

use crate::maybe_starts_with_dollar::MaybeStartsWithDollar;

#[derive(Default)]
pub(crate) struct PathAttribute {
    macro_v: Option<MaybeStartsWithDollar<Path>>,
    nade_helper: Option<MaybeStartsWithDollar<Path>>,
}

impl PathAttribute {
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

        Ok(())
    }

    pub(crate) fn simplify(self) -> SimplePathAttribute {
        let PathAttribute {
            macro_v,
            nade_helper,
        } = self;

        SimplePathAttribute {
            macro_v: macro_v.unwrap_or_else(|| parse_quote!(::nade::__internal)),
            nade_helper: nade_helper.unwrap_or_else(|| parse_quote!($crate)),
        }
    }
}

impl TryFrom<&mut Vec<Attribute>> for PathAttribute {
    type Error = syn::Error;

    fn try_from(attrs: &mut Vec<Attribute>) -> Result<Self, Self::Error> {
        let mut path_attribute = PathAttribute::default();
        let mut errors = Vec::new();

        attrs.retain(|attr| {
            if !attr.path().is_ident("nade_path") {
                return true;
            }

            if let Err(e) = attr.parse_nested_meta(|meta| path_attribute.parse_meta(meta)) {
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

        Ok(path_attribute)
    }
}

pub(crate) struct SimplePathAttribute {
    pub(crate) macro_v: MaybeStartsWithDollar<Path>,
    pub(crate) nade_helper: MaybeStartsWithDollar<Path>,
}
