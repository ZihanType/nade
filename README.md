# nade

[![Crates.io version](https://img.shields.io/crates/v/nade.svg?style=flat-square)](https://crates.io/crates/nade)
[![docs.rs docs](https://img.shields.io/badge/docs-latest-blue.svg?style=flat-square)](https://docs.rs/nade)

`nade` is a attribute macro that adds ***na***med and ***de***fault arguments to Rust functions.

## Usage

```rust
// some_crate/src/lib.rs
pub use nade::core::*;
use nade::nade;

pub fn one() -> u32 {
    1
}

#[nade]
pub fn foo(
    /// You can add doc comments to the parameter. It will be shown in the doc of the macro.
    /// The world is 42.
    #[nade(42)] a: u32,

    /// Call a function
    #[nade(one())] b: u32,

    /// Default value of u32
    #[nade] c: u32,

    d: u32
) -> u32 {
    a + b + c + d
}

assert_eq!(foo!(1, 2, 3, 4), 10); // foo(1, 2, 3, 4)
assert_eq!(foo!(d = 2), 45); // foo(42, one(), Default::default(), 2)
assert_eq!(foo!(1, c = 2, b = 3, 4), 10); // foo(1, 3, 2, 4)
```

## How it works

If you write a function like this:

```rust
pub fn one() -> u32 {
    1
}

#[nade]
pub fn foo(
    #[nade(42)]
    a: u32,

    #[nade(one())]
    b: u32,

    #[nade]
    c: u32,

    d: u32
) -> u32 {
    a + b + c + d
}
```

it will be expanded to:

```rust
pub fn one() -> u32 {
    1
}

pub fn foo(a: u32, b: u32, c: u32, d: u32) -> u32 {
    a + b + c + d
}

#[crate::macro_v(pub)]
macro_rules! foo {
    ($($args:tt)*) => {
        $crate::nade_helper!(
            ($($args)*)
            (a = 42, b = one(), c = Default::default())
            (foo)
        )
    };
}
```

The attribute macro `#[macro_v(pub)]` make the visibility of the declarative macro the same as the function. When the visibility of the function is `pub(crate)`, `#[macro_v(pub(crate))]` is also generated. see [macro-v](https://github.com/ZihanType/macro-v) for details.

Then, when you call the macro `foo` like this:

```rust
foo!(32, d = 1, c = 2);
```

it will be expanded to:

```rust
foo(32, one(), 2, 1);
```

## Note

As you can see in [How it works](#how-it-works), the code generated by `#[nade]` contains calls to the macros `#[crate::macro_v(pub)]` and `$crate::nade_helper!(..)`, so you have to import them in the root of crate, because the declarative macro `foo` may be used in other crate so it is recommended to use the `pub use` statement.

```rust
// recommend
pub use nade::core::*;
// or expand glob import
pub use nade::core::{macro_v, nade_helper};
```

## Limitations

1. When you call the macro `foo`, you must use the `use` statement to bring the macro into scope.

    ```rust
    // Good
    use some_crate::foo;
    foo!(32, d = 1, c = 2);

    // Bad
    some_crate::foo!(32, d = 1, c = 2);
    ```

    Because the attribute macro `nade` will generate a macro with the same name as the function, and the macro use the function, so you must use the `use` statement to bring the macro and the function into scope.

2. The default argument expression must be declared in the scope of the macro call.

    ```rust
    // Good
    use some_crate::one;
    foo!(32, d = 1, c = 2);

    // Bad
    foo!(32, d = 1, c = 2);
    ```

    Because the default argument expression is evaluated after the macro is expanded, so it must be declared in the scope of the macro call.

## How to bypass the limitations

1. You can pass a module path starting with `$crate` for the `nade` attribute macro on the function.

    ```rust
    #[nade($crate::module)]
    pub fn foo(
        #[nade(42)]
        a: u32,

        #[nade(one())]
        b: u32,

        #[nade]
        c: u32,

        d: u32
    ) -> u32 {
        a + b + c + d
    }
    ```

    it will be expanded to:

    ```rust
    pub fn foo(a: u32, b: u32, c: u32, d: u32) -> u32 {
        a + b + c + d
    }

    #[crate::macro_v(pub)]
    macro_rules! foo {
        ($($args:tt)*) => {
            $crate::nade_helper!(
                ($($args)*)
                (a = 42, b = one(), c = Default::default())
                ($crate::module::foo)
            )
        };
    }
    ```

    Then, you can not use the `use` statement to bring the macro into scope, like this:

    ```rust
    some_crate::foo!(32, d = 1, c = 2);
    ```

2. In the `nade` attribute macro on the parameter, you can specify the default argument expression using the full path, either `$crate::a::expr`, or `::a::b::expr`. In fact, when you use `#[nade]` on an parameter, you are using `#[nade(::core::default::Default::default())]`.

    ```rust
    pub fn one() -> u32 {
        1
    }

    pub static PATH: &str = "a";

    #[nade]
    pub fn foo<T1, T2, T3, T4>(
        #[nade($crate::module::one())]
        a: T1,

        #[nade(::std::path::Path::new("a"))]
        b: T2,

        #[nade($crate::module::PATH)]
        c: T3,

        #[nade("Hello")]
        d: T4
    ) {
        let _ = (a, b, c, d);
    }
    ```

    it will be expanded to:

    ```rust
    pub fn foo<T1, T2, T3, T4>(a: T1, b: T2, c: T3, d: T4) {
        let _ = (a, b, c, d);
    }

    #[crate::macro_v(pub)]
    macro_rules! foo {
        ($($args:tt)*) => {
            $crate::nade_helper!(
                ($($args)*)
                (
                    a = $crate::module::one(),
                    b = ::std::path::Path::new("a"),
                    c = $crate::module::PATH,
                    d = "Hello"
                )
                (foo)
            )
        };
    }
    ```

    Then, you can not use the `use` statement to bring default argument expressions into scope, like this:

    ```rust
    foo!();
    ```

## Credits

This crate is inspired by these crates:

- [default-args](https://github.com/buttercrab/default-args.rs)
- [duang](https://github.com/xiaoniu-578fa6bff964d005/duang)
- [leptos](https://github.com/gbj/leptos)
- [typed-builder](https://github.com/idanarye/rust-typed-builder)
