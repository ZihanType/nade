use nade::nade;

#[test]
fn no_paramter() {
    #[nade]
    fn foo() -> u32 {
        1
    }

    assert_eq!(foo!(), 1);
}

#[test]
fn one_default_argument() {
    #[nade]
    fn foo(#[nade] a: u32) -> u32 {
        a
    }

    assert_eq!(foo!(), 0);
    assert_eq!(foo!(1), 1);
    assert_eq!(foo!(a = 1), 1);
}

#[test]
fn mix_default_and_not_default_arguments() {
    #[nade]
    fn foo(#[nade(42)] a: u32, b: u32, #[nade] c: u32, d: u32) -> u32 {
        a + b + c + d
    }

    assert_eq!(foo!(b = 2, d = 4), 48);
    assert_eq!(foo!(1, 2, 3, 4), 10);
    assert_eq!(foo!(a = 1, 2, c = 3, 4), 10);
    assert_eq!(foo!(c = 1, 1, d = 2), 46);
}

#[test]
fn all_default_arguments() {
    #[nade]
    fn foo(#[nade(42)] a: u32, #[nade] b: u32, #[nade] c: u32, #[nade] d: u32) -> u32 {
        a + b + c + d
    }

    assert_eq!(foo!(), 42);
}

#[test]
fn all_not_default_arguments() {
    #[nade]
    fn foo(a: u32, b: u32, c: u32, d: u32) -> u32 {
        a + b + c + d
    }

    assert_eq!(foo!(a = 1, b = 2, c = 3, d = 4), 10);
    assert_eq!(foo!(c = 1, 2, a = 3, 4), 10);
    assert_eq!(foo!(b = 1, c = 2, a = 3, 4), 10);
    assert_eq!(foo!(1, 2, 3, 4), 10);
}

#[test]
fn generic() {
    #[nade]
    fn foo<T: AsRef<str>>(#[nade("hello")] a: T) -> String {
        a.as_ref().to_string()
    }

    assert_eq!(foo!(), "hello");
    assert_eq!(foo!("world"), "world");
    assert_eq!(foo!(a = "a"), "a");
    assert_eq!(foo!(a = String::from("abcd")), "abcd");
}

#[test]
fn pattern_matching() {
    pub struct One<T>(T);

    #[nade]
    pub fn foo(#[nade((One(1), Some(2)))] (One(a), _): (One<u32>, Option<u32>)) -> u32 {
        a
    }

    assert_eq!(foo!(), 1);
    assert_eq!(foo!((One(a), _) = (One(2), Some(3))), 2);
    assert_eq!(foo!((One(3), None)), 3);
}
