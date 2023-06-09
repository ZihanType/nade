# Changelog

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
