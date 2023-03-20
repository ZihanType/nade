use nade::macro_v;
pub use nade::nade_helper;

mod foo {
    use nade::nade;

    #[nade]
    pub fn bar() -> usize {
        1
    }
}

fn main() {
    assert_eq!(foo::bar!(), 1);
}
