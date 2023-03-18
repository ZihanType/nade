use syn::{Expr, LitStr, Pat, Type};

pub(crate) struct ParameterDoc {
    pub(crate) pattern: Pat,
    pub(crate) ty: Type,
    pub(crate) docs: Vec<LitStr>,
    pub(crate) default: Option<Expr>,
}
