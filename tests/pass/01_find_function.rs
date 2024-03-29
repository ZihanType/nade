pub use nade::base::*;

mod foo1 {
    use nade::nade;

    #[nade]
    pub fn bar() -> usize {
        1
    }
}

mod foo2 {
    use nade::nade;

    #[nade(module_path = $crate::foo2)]
    pub fn bar() -> usize {
        1
    }
}

fn main() {
    {
        // import function and macro
        use foo1::bar;
        assert_eq!(bar!(), 1);
    }

    {
        // only import function
        use foo1::bar;
        assert_eq!(foo1::bar!(), 1);
    }

    {
        // specify the module path, so there is no need to import function
        assert_eq!(foo2::bar!(), 1);
    }
}
