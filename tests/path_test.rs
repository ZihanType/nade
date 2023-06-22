pub use nade::base::*;

pub mod foo {
    use nade::nade;

    #[nade($crate::foo)]
    pub fn bar(#[nade($crate::foo::baz())] a: usize) -> usize {
        a
    }

    pub fn baz() -> usize {
        1
    }
}

use nade::nade;

#[nade($crate)]
pub fn bar(#[nade($crate::baz())] a: usize) -> usize {
    a
}

pub fn baz() -> usize {
    1
}

#[test]
fn path_test() {
    assert_eq!(bar!(), 1);
    assert_eq!(foo::bar!(), 1);
}
