pub use nade::base::*;

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
