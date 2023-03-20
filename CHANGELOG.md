# Changelog

## [0.2.1] 2023-03-20

- Internal change: move internal macros to `core` mod.

## [0.2.0] 2023-03-20

- Fix: use import macros in the root of crate instead of creating a mod and re-exporting it.

## [0.1.1] 2023-03-19

- Fix: in `#[nade(...)]` , it is not necessary that `$crate` must be followed by `::`.

## [0.1.0] 2023-03-18

- Add `#[nade]`.
