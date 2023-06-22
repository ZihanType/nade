#![doc = include_str!("../README.md")]

pub use nade_macro::nade;

#[doc(hidden)]
pub mod base {
    pub use nade_macro::nade_helper;
}

#[doc(hidden)]
pub mod __internal {
    pub use macro_v::macro_v;
}
