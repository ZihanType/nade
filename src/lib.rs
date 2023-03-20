#![doc = include_str!("../README.md")]

pub use nade_macro::nade;

#[doc(hidden)]
pub use crate::core::*;

pub mod core {
    #[doc(hidden)]
    pub use macro_v::macro_v;
    #[doc(hidden)]
    pub use nade_macro::nade_helper;
}
