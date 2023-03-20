pub use nade::nade_helper;

pub mod foo {
    use nade::macro_v;

    pub fn bar(a: u32, b: u32, c: u32) -> u32 {
        a + b + c
    }

    #[macro_v(pub)]
    macro_rules! baz {
        ($($arg:tt)*) => {
            $crate::nade_helper!(
                ($($arg)*)
                (a, b = $crate::foo::aaa(), c = 4)
                ($crate::foo::bar)
            )
        };
    }

    pub fn aaa() -> u32 {
        2
    }
}

#[test]
fn manual_test() {
    assert_eq!(foo::baz!(a = 1), 7);
}
