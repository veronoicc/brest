# Unreleased

## Added

- `?` error API: every `std::error::Error` converts into `BrestErr` (status 500, `type: "error"`), so handlers returning `Result<Brest<T>, BrestErr>` can use `?` on any fallible call.
- `ResultExt` with `.status(..)`, `.code(..)`, `.fail()` and `.message(..)` to override the response at the call site; they chain.
- `BrestErr::{error, fail, with_status, with_code, with_message, into_fail, into_error}`, `Brest::{with_status, with_code}`.
- `IntoResponse` for `BrestErr`.
- Own error types that do not implement `std::error::Error` can `impl From<MyErr> for BrestErr` to choose their response.
- With the nightly `try` feature, handlers may return `Brest<T>` directly and use `?` on `Result`s and other `Brest`s.

## Changed (breaking, relative to 0.2.2)

- `BrestErr` no longer implements `std::error::Error` (required for the blanket `From<E: Error>` conversion) and is no longer gated on the `axum` feature (its `status` field still is).
- `schemars` feature now uses schemars 1.x (was 0.8).
- Edition 2024, MSRV 1.85 (was undeclared; `[package.metadata] msrv` removed).
- `try` feature: `Try::Residual` for `Brest<D, C>` is now `BrestErr<C>` (was `Brest<(), C>`), which fixes compilation on current nightly. The `FromResidual` impls for `Result<D, E>`, `(Result<D, E>, C)`, `(Result<D, E>, C, S)` and `Result<U, Brest>` were removed; use `Result<_, E>` with `?` or `ResultExt` instead.
- docs.rs builds with all features.
