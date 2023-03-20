pub use nade::core::*;
use nade::nade;

pub mod foo {
    use super::*;

    #[nade($crate::foo)]
    pub fn bar(#[nade($crate::foo::baz())] a: usize) -> usize {
        a
    }

    pub fn baz() -> usize {
        1
    }
}

#[test]
fn path_test() {
    assert_eq!(foo::bar!(), 1);
}
