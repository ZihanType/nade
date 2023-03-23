use syn::{
    parse::{Parse, ParseStream},
    Expr, Pat, Token,
};

pub(crate) enum Argument {
    Positioned { value: Expr },
    Named { pattern: Pat, value: Expr },
}

impl Parse for Argument {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let test = input.fork();
        let argument = if test.call(Pat::parse_single).is_ok() && test.peek(Token![=]) {
            let pattern = input.call(Pat::parse_single)?;
            input.parse::<Token![=]>()?;
            let value = input.parse::<Expr>()?;
            Argument::Named { pattern, value }
        } else {
            let value = input.parse::<Expr>()?;
            Argument::Positioned { value }
        };

        Ok(argument)
    }
}
