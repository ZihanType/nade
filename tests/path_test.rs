pub use nade::base::*;

pub mod foo {
    use nade::nade;

    #[nade(module_path = $crate::foo)]
    pub fn bar(#[nade($crate::foo::baz())] a: usize) -> usize {
        a
    }

    pub fn baz() -> usize {
        1
    }
}

use nade::nade;

#[nade(module_path = $crate)]
pub fn bar(#[nade($crate::baz())] a: usize) -> usize {
    a
}

pub fn baz() -> usize {
    1
}

#[nade]
#[nade_path(macro_v = ::nade::__internal, nade_helper = $crate)]
fn default_path(a: usize) -> usize {
    a
}

mod custom_macro_v {
    pub use macro_v::macro_v;
}

#[nade]
#[nade_path(macro_v = custom_macro_v)]
fn custom_macro_v_path(a: usize) -> usize {
    a
}

mod custom_nade_helper {
    pub use nade_macro::nade_helper;
}

#[nade]
#[nade_path(nade_helper = custom_nade_helper)]
fn custom_nade_helper_path(a: usize) -> usize {
    a
}

#[nade]
#[nade_path(macro_v = custom_macro_v)]
#[nade_path(nade_helper = custom_nade_helper)]
fn custom_path(a: usize) -> usize {
    a
}

#[test]
fn path_test() {
    assert_eq!(bar!(), 1);
    assert_eq!(foo::bar!(), 1);
    assert_eq!(default_path!(1), 1);
    assert_eq!(custom_macro_v_path!(1), 1);
    assert_eq!(custom_nade_helper_path!(1), 1);
    assert_eq!(custom_path!(1), 1);
}
