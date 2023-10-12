# nade

[![Crates.io version](https://img.shields.io/crates/v/nade.svg?style=flat-square)](https://crates.io/crates/nade)
[![docs.rs docs](https://img.shields.io/badge/docs-latest-blue.svg?style=flat-square)](https://docs.rs/nade)

[English](./README.md) | 简体中文

`nade`是一个为Rust函数添加命名参数(***na***med)和默认参数(***de***fault)的属性宏。

## 用法

```rust
// some_crate/src/lib.rs
pub use nade::base::*;
use nade::nade;

pub fn one() -> u32 {
    1
}

#[nade]
pub fn foo(
    /// 可以为参数添加文档注释。它将显示在宏的文档中。
    ///
    /// 宇宙的答案是42。
    #[nade(42)] a: u32,

    /// 使用函数的返回值作为默认值
    #[nade(one())] b: u32,

    /// 使用u32的默认值
    #[nade] c: u32,

    d: u32
) -> u32 {
    a + b + c + d
}

assert_eq!(foo!(1, 2, 3, 4), 10);         // foo(1,  2,     3,                  4)
assert_eq!(foo!(d = 2), 45);              // foo(42, one(), Default::default(), 2)
assert_eq!(foo!(1, c = 2, b = 3, 4), 10); // foo(1,  3,     2,                  4)
```

## 原理

如果你写了这样一个函数：

```rust
// some_crate/src/lib.rs
pub use nade::base::*;
use nade::nade;

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

它会被展开为：

```rust
// some_crate/src/lib.rs
// ⓵
pub use nade::base::*;
use nade::nade;

pub fn one() -> u32 {
    1
}

pub fn foo(a: u32, b: u32, c: u32, d: u32) -> u32 {
    a + b + c + d
}

// ⓶
#[::nade::__internal::macro_v(pub)]
macro_rules! foo {
    ($($arguments:tt)*) => {
        // ⓷
        $crate::nade_helper!(
            ($($arguments)*)
            (a: u32 = 42, b: u32 = one(), c: u32 = Default::default(), d: u32)
            (foo)
        )
    };
}
```

然后，当你像这样调用宏`foo`：

```rust
use some_crate::{foo, one};

foo!(32, d = 1, c = 2);
```

它会被展开为：

```rust
use some_crate::{foo, one};

foo(32, one(), 2, 1);
```

### 注意

正如你在上面的[原理](#原理)中看到的，生成的代码中有3个地方需要注意。

- ⓵, ⓷

    `nade_helper`是一个基于实参、行参、函数路径生成函数调用表达式的声明式宏。

    它的默认路径是`$crate::nade_helper`，所以你必须在crate的根目录中导入它，用`pub use nade::base::*;`或者`pub use nade::base::nade_helper;`。

    你也可以自定义`nade_helper`的路径。

    ```rust
    use nade::nade;

    mod custom_nade_helper {
        pub use nade::base::nade_helper;
    }

    #[nade]
    #[nade_path(nade_helper = custom_nade_helper)]
    fn custom_nade_helper_path(a: usize) -> usize {
        a
    }
    ```

- ⓶

    `macro_v`是一个使声明式宏的可见性和函数一样的属性宏。可以在[macro-v](https://github.com/ZihanType/macro-v)看到更详细的信息。

    它的默认路径是`::nade::__internal::macro_v`。

    你也可以自定义`macro_v`的路径。

    ```rust
    use nade::nade;

    mod custom_macro_v {
        pub use nade::__internal::macro_v;
    }

    #[nade]
    #[nade_path(macro_v = custom_macro_v)]
    fn custom_macro_v_path(a: usize) -> usize {
        a
    }
    ```

## 限制

1. 当你调用`foo`宏的时候，你必须用`use`语句将`foo`导入到作用域中。

    ```rust
    // 得这样
    use some_crate::{foo, one};
    foo!(32, d = 1, c = 2);

    // 而不能这样
    use some_crate::one;
    some_crate::foo!(32, d = 1, c = 2);
    ```

    因为属性宏`#[nade]`会生成一个和被标记的函数同名的宏，这个宏会以不卫生的方式使用到该函数，所以你必须用`use`语句将宏**和**函数都导入到作用域中。

2. 默认参数表达式必须被导入到宏调用的作用域中。

    ```rust
    // 得这样
    use some_crate::{foo, one};
    foo!(32, d = 1, c = 2);

    // 而不能这样
    use some_crate::foo;
    foo!(32, d = 1, c = 2);
    ```

    因为默认参数表达式是在`foo`宏展开后，才求值的，所以必须将表达式导入到宏调用的作用域中。

## 如何绕开限制

1. 你可以传递一个以`$crate`开头的模块路径给标记在函数上的`#[nade]`属性宏。

    ```rust
    #[nade(module_path = $crate::module)] // <--- 注意看这
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

    它将会展开为：

    ```rust
    pub fn foo(a: u32, b: u32, c: u32, d: u32) -> u32 {
        a + b + c + d
    }

    #[::nade::__internal::macro_v(pub)]
    macro_rules! foo {
        ($($arguments:tt)*) => {
            $crate::nade_helper!(
                ($($arguments)*)
                (a: u32 = 42, b: u32 = one(), c: u32 = Default::default(), d: u32)
                ($crate::module::foo) // <--- 注意看这
            )
        };
    }
    ```

    然后，就不用使用`use`语句将宏和函数都导入到作用域中了，像这样：

    ```rust
    use some_crate::one;
    some_crate::foo!(32, d = 1, c = 2);
    ```

2. 对标记在参数上的`#[nade]`属性宏，你可以指定默认参数表达式的全路径，比如`$crate::a::expr`或者`::a::b::expr`。事实上，当你在参数上使用`#[nade]`的时候，实际上是使用了`#[nade(::core::default::Default::default())]`。

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

    它将会展开为：

    ```rust
    pub fn foo<T1, T2, T3, T4>(a: T1, b: T2, c: T3, d: T4) {
        let _ = (a, b, c, d);
    }

    #[::nade::__internal::macro_v(pub)]
    macro_rules! foo {
        ($($arguments:tt)*) => {
            $crate::nade_helper!(
                ($($arguments)*)
                (
                    a: T1 = $crate::module::one(),
                    b: T2 = ::std::path::Path::new("a"),
                    c: T3 = $crate::module::PATH,
                    d: T4 = "Hello",
                )
                (foo)
            )
        };
    }
    ```

    然后，就不用使用`use`语句将默认参数表达式导入到宏调用的作用域中了，像这样：

    ```rust
    use some_crate::foo;
    foo!();
    ```

## 鸣谢

这个crate受到了这些crate的启发：

- [default-args](https://github.com/buttercrab/default-args.rs)
- [duang](https://github.com/xiaoniu-578fa6bff964d005/duang)
- [leptos](https://github.com/leptos-rs/leptos)
- [typed-builder](https://github.com/idanarye/rust-typed-builder)
