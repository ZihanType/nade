# Changelog

## [0.3.3] 2023-10-12

### Breaking

- add attribute argument name module_path.

    originally written like this:

    ```rust
    #[nade($crate)]
    fn a() {}
    ```

    should be changed to:

    ```rust
    #[nade(module_path = $crate)]
    fn a() {}
    ```

### New Features

- add attribute arguments that customize the paths of the `macro_v` and `nade_helper` macros.
- add return type doc.

### Fixes

- change the heading level of documents with generated parameters and return type.

### Internal Improvements

- refactor parse methods of `MaybeStartsWithDollar` and `StartsWithDollar`.
- improving the accuracy of error messages.
- reduce clone method calls.

### Docs

- update README.md.
- add README-zh_cn.md.

## [0.3.2] 2023-08-05

- feat: Relaxing constraints on default value expression types.

## [0.3.1] 2023-07-29

- doc: Update README.MD.
- chore: Remove unnecessary internal struct.
- fix: Improved error message when a parameter is specified multiple times by named.
- fix: Improved error message when `#[nade(..)]` is used multiple times on a single parameter.
- chore: Rename internal function name.
- chore: Simplified implementation of `Parse` trait for `Parameter` struct.
- fix: Recover some of syn's features.

## [0.3.0] 2023-06-22

- chore: modify `Option<TokenStream>` to `TokenStream`.
- chore: fix stderr files.
- chore: fix `Leptos` link in README.md.
- chore: add `default-features = false` in Cargo.toml.
- rename: `core` mod to `base`.
- fix: `#[nade(...)]` now expand to `::nade::__internal::macro_v(...)` instead of `crate::macro_v(...)`.

## [0.2.1] 2023-03-20

- Internal change: move internal macros to `core` mod.

## [0.2.0] 2023-03-20

- Fix: use import macros in the root of crate instead of creating a mod and re-exporting it.

## [0.1.1] 2023-03-19

- Fix: in `#[nade(...)]` , it is not necessary that `$crate` must be followed by `::`.

## [0.1.0] 2023-03-18

- Add `#[nade]`.
