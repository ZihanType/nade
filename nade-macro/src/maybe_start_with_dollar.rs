use proc_macro2::TokenStream;
use quote::ToTokens;
use syn::{
    parse::{Parse, ParseStream},
    Token,
};

pub(crate) enum MaybeStartWithDollar<T> {
    StartWithDollar(StartWithDollar<T>),
    Normal(T),
}

impl<T: Parse> Parse for MaybeStartWithDollar<T> {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        Ok(if input.peek(Token![$]) {
            Self::StartWithDollar(input.parse()?)
        } else {
            Self::Normal(input.parse()?)
        })
    }
}

impl<T: ToTokens> ToTokens for MaybeStartWithDollar<T> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        match self {
            MaybeStartWithDollar::StartWithDollar(e) => e.to_tokens(tokens),
            MaybeStartWithDollar::Normal(e) => e.to_tokens(tokens),
        }
    }
}

impl<T> MaybeStartWithDollar<T> {
    pub(crate) fn inner(&self) -> &T {
        match self {
            MaybeStartWithDollar::StartWithDollar(s) => &s.inner,
            MaybeStartWithDollar::Normal(n) => n,
        }
    }

    pub(crate) fn as_ref(&self) -> MaybeStartWithDollar<&T> {
        match self {
            MaybeStartWithDollar::StartWithDollar(StartWithDollar {
                dollar_token,
                inner,
            }) => MaybeStartWithDollar::StartWithDollar(StartWithDollar {
                dollar_token: *dollar_token,
                inner,
            }),
            MaybeStartWithDollar::Normal(n) => MaybeStartWithDollar::Normal(n),
        }
    }
}

pub(crate) struct StartWithDollar<T> {
    dollar_token: Token![$],
    inner: T,
}

impl<T: Parse> Parse for StartWithDollar<T> {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let dollar_token = input.parse::<Token![$]>()?;

        let lookahead = input.lookahead1();

        if lookahead.peek(Token![crate]) {
            Ok(Self {
                dollar_token,
                inner: input.parse()?,
            })
        } else {
            Err(lookahead.error())
        }
    }
}

impl<T: ToTokens> ToTokens for StartWithDollar<T> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        self.dollar_token.to_tokens(tokens);
        self.inner.to_tokens(tokens);
    }
}
