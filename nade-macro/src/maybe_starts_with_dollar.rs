use proc_macro2::{Punct, TokenStream};
use quote::ToTokens;
use syn::{
    parse::{Parse, ParseStream},
    spanned::Spanned,
    Token,
};

#[derive(Clone)]
pub(crate) enum MaybeStartsWithDollar<T> {
    StartsWithDollar(StartsWithDollar<T>),
    Normal(Normal<T>),
}

impl<T: Parse> Parse for MaybeStartsWithDollar<T> {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let dollar_input = input.fork();
        if let Some(punct) = dollar_input.parse::<Option<Punct>>()? {
            if '$' == punct.as_char() {
                return Ok(Self::StartsWithDollar(StartsWithDollar::parse(
                    &dollar_input,
                )?));
            }
        }

        Ok(Self::Normal(Normal::parse(input)?))
    }
}

impl<T: ToTokens> ToTokens for MaybeStartsWithDollar<T> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        match self {
            MaybeStartsWithDollar::StartsWithDollar(e) => e.to_tokens(tokens),
            MaybeStartsWithDollar::Normal(e) => e.to_tokens(tokens),
        }
    }
}

impl<T: Parse> TryFrom<proc_macro::TokenStream> for MaybeStartsWithDollar<T> {
    type Error = syn::Error;

    fn try_from(value: proc_macro::TokenStream) -> Result<Self, Self::Error> {
        let mut iter = value.into_iter().peekable();

        if let Some(proc_macro::TokenTree::Punct(p)) = iter.peek() {
            if p.as_char() == '$' {
                let s = syn::parse::<StartsWithDollar<T>>(iter.skip(1).collect())?;
                return Ok(MaybeStartsWithDollar::StartsWithDollar(s));
            }
        }

        let n = syn::parse::<Normal<T>>(iter.collect())?;
        Ok(MaybeStartsWithDollar::Normal(n))
    }
}

impl<T> MaybeStartsWithDollar<T> {
    pub(crate) fn inner(&self) -> &T {
        match self {
            MaybeStartsWithDollar::StartsWithDollar(s) => &s.inner,
            MaybeStartsWithDollar::Normal(n) => &n.inner,
        }
    }
}

impl<T: ToTokens> MaybeStartsWithDollar<T> {
    pub(crate) fn starts_with_dollar(self) -> syn::Result<StartsWithDollar<T>> {
        match self {
            MaybeStartsWithDollar::StartsWithDollar(s) => Ok(s),
            MaybeStartsWithDollar::Normal(n) => Err(syn::Error::new(
                n.inner.span(),
                "expected starting with `$crate`",
            )),
        }
    }
}

#[derive(Clone)]
pub(crate) struct StartsWithDollar<T> {
    pub(crate) inner: T,
}

impl<T: Parse> Parse for StartsWithDollar<T> {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        if input.peek(Token![crate]) {
            Ok(Self {
                inner: input.parse()?,
            })
        } else {
            Err(input.error("expected starting with `$crate`"))
        }
    }
}

impl<T: ToTokens> ToTokens for StartsWithDollar<T> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        <Token![$]>::default().to_tokens(tokens);
        self.inner.to_tokens(tokens);
    }
}

#[derive(Clone)]
pub(crate) struct Normal<T> {
    pub(crate) inner: T,
}

impl<T: Parse> Parse for Normal<T> {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        Ok(Self {
            inner: input.parse()?,
        })
    }
}

impl<T: ToTokens> ToTokens for Normal<T> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        self.inner.to_tokens(tokens);
    }
}
