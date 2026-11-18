//! Simple response format for web apis.
//!
//! Every response is one of `{"type":"success","data":..}`, `{"type":"error","message":..}` or
//! `{"type":"fail","message":..}` (see [`Brest`]).
//!
//! # Features
//!
//! - `axum`: `IntoResponse` for [`Brest`] and [`BrestErr`], HTTP statuses, and the
//!   `extractors` module.
//! - `schemars`: `JsonSchema` for [`Brest`] (schemars 1.x).
//! - `try`: nightly only. Implements the `?` operator for [`Brest`], so handlers may return
//!   `Brest<T>` directly. Never enable it on stable.
//!
//! # Using `?` in handlers
//!
//! Handlers returning `Result<Brest<T>, BrestErr>` accept `?` on any [`std::error::Error`]
//! (answers `500` with `{"type":"error","message":..}`):
//!
//! ```
//! # #[cfg(feature = "axum")]
//! # mod example {
//! use brest::{Brest, BrestErr};
//!
//! async fn handler() -> Result<Brest<String>, BrestErr> {
//!     let text = std::fs::read_to_string("greeting.txt")?;
//!     Ok(Brest::success(text))
//! }
//! # }
//! # fn main() {}
//! ```
//!
//! Errors that do not implement [`std::error::Error`] (`Box<dyn Error + Send + Sync>`,
//! `anyhow::Error`, `String`) cannot convert implicitly; use `.map_err(BrestErr::error)?`.
//!
//! Change status, code or kind at the call site with [`ResultExt`]:
//!
//! ```
//! # #[cfg(feature = "axum")]
//! # mod example {
//! use axum::http::StatusCode;
//! use brest::{Brest, BrestErr, ResultExt as _};
//!
//! async fn handler() -> Result<Brest<String>, BrestErr> {
//!     let missing = std::fs::read_to_string("greeting.txt").status(StatusCode::NOT_FOUND)?;
//!     let bad_input = std::fs::read_to_string("other.txt").fail().code(42)?;
//!     Ok(Brest::success(missing + &bad_input))
//! }
//! # }
//! # fn main() {}
//! ```
//!
//! Your own error types that do not implement [`std::error::Error`] pick their own response
//! with `impl From<MyErr> for BrestErr`:
//!
//! ```
//! # #[cfg(feature = "axum")]
//! # mod example {
//! use axum::http::StatusCode;
//! use brest::{Brest, BrestErr};
//!
//! struct OutOfStock;
//!
//! impl From<OutOfStock> for BrestErr {
//!     fn from(_: OutOfStock) -> Self {
//!         BrestErr::fail("out of stock").with_status(StatusCode::CONFLICT)
//!     }
//! }
//!
//! async fn handler() -> Result<Brest<()>, BrestErr> {
//!     Err(OutOfStock)?
//! }
//! # }
//! # fn main() {}
//! ```
//!
//! With the nightly `try` feature the same `?` works in a handler returning [`Brest`] itself.
#![cfg_attr(feature = "try", feature(try_trait_v2, try_trait_v2_residual))]

#[cfg(feature = "try")]
use std::{
    convert::Infallible,
    ops::{ControlFlow, FromResidual, Residual, Try},
};

#[cfg(feature = "axum")]
use axum::{http::StatusCode, response::IntoResponse};

#[cfg(feature = "schemars")]
use schemars::JsonSchema;

#[cfg(feature = "axum")]
use serde::{Serializer, ser::SerializeStruct as _};

use serde::{Deserialize, Serialize};

use std::fmt::Debug;

#[cfg(feature = "axum")]
pub mod extractors;

#[derive(Debug)]
#[cfg_attr(feature = "schemars", derive(JsonSchema))]
#[derive(Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase", tag = "type")]
pub enum Brest<D: Serialize = (), C = u32> {
    Success {
        data: D,
        #[cfg(feature = "axum")]
        #[serde(skip)]
        status: StatusCode,
    },
    Error {
        message: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        code: Option<C>,
        #[cfg(feature = "axum")]
        #[serde(skip)]
        status: StatusCode,
    },
    Fail {
        message: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        code: Option<C>,
        #[cfg(feature = "axum")]
        #[serde(skip)]
        status: StatusCode,
    },
}

impl<D: Serialize, C> Brest<D, C> {
    pub fn success(data: D) -> Self {
        Self::Success {
            data,
            #[cfg(feature = "axum")]
            status: StatusCode::OK,
        }
    }

    #[cfg(feature = "axum")]
    pub fn success_status(data: D, status: StatusCode) -> Self {
        Self::Success { data, status }
    }

    pub fn error<M: ToString>(message: M) -> Self {
        Self::Error {
            message: message.to_string(),
            code: None,
            #[cfg(feature = "axum")]
            status: StatusCode::INTERNAL_SERVER_ERROR,
        }
    }

    pub fn error_code<M: ToString>(message: M, code: C) -> Self {
        Self::Error {
            message: message.to_string(),
            code: Some(code),
            #[cfg(feature = "axum")]
            status: StatusCode::INTERNAL_SERVER_ERROR,
        }
    }

    #[cfg(feature = "axum")]
    pub fn error_status<M: ToString>(message: M, status: StatusCode) -> Self {
        Self::Error {
            message: message.to_string(),
            code: None,
            #[cfg(feature = "axum")]
            status,
        }
    }

    #[cfg(feature = "axum")]
    pub fn error_code_status<M: ToString>(message: M, code: C, status: StatusCode) -> Self {
        Self::Error {
            message: message.to_string(),
            code: Some(code),
            status,
        }
    }

    pub fn fail<M: ToString>(message: M) -> Self {
        Self::Fail {
            message: message.to_string(),
            code: None,
            #[cfg(feature = "axum")]
            status: StatusCode::BAD_REQUEST,
        }
    }

    pub fn fail_code<M: ToString>(message: M, code: C) -> Self {
        Self::Fail {
            message: message.to_string(),
            code: Some(code),
            #[cfg(feature = "axum")]
            status: StatusCode::BAD_REQUEST,
        }
    }

    #[cfg(feature = "axum")]
    pub fn fail_status<M: ToString>(message: M, status: StatusCode) -> Self {
        Self::Fail {
            message: message.to_string(),
            code: None,
            status,
        }
    }

    #[cfg(feature = "axum")]
    pub fn fail_code_status<M: ToString>(message: M, code: C, status: StatusCode) -> Self {
        Self::Fail {
            message: message.to_string(),
            code: Some(code),
            #[cfg(feature = "axum")]
            status,
        }
    }
}

impl<D: Serialize, C> Brest<D, C> {
    /// Sets the HTTP status (any variant).
    #[cfg(feature = "axum")]
    #[must_use]
    pub fn with_status(mut self, status: StatusCode) -> Self {
        let (Self::Success { status: s, .. }
        | Self::Error { status: s, .. }
        | Self::Fail { status: s, .. }) = &mut self;
        *s = status;
        self
    }

    /// Sets the application code. No-op on `Success`, which carries none.
    #[must_use]
    pub fn with_code(mut self, code: C) -> Self {
        if let Self::Error { code: c, .. } | Self::Fail { code: c, .. } = &mut self {
            *c = Some(code);
        }
        self
    }
}

impl<D: Serialize, C> Brest<D, C> {
    #[inline]
    #[must_use]
    pub fn is_success(&self) -> bool {
        matches!(self, Self::Success { .. })
    }

    #[inline]
    #[must_use]
    pub fn is_success_and<F: FnOnce(D) -> bool>(self, f: F) -> bool {
        match self {
            Self::Success { data, .. } => f(data),
            _ => false,
        }
    }

    #[inline]
    #[must_use]
    pub fn is_fail(&self) -> bool {
        matches!(self, Self::Fail { .. })
    }

    #[inline]
    #[must_use]
    pub fn is_fail_and<F: FnOnce(ErrorFields<C>) -> bool>(self, f: F) -> bool {
        match self {
            #[cfg(feature = "axum")]
            Self::Fail {
                message,
                code,
                status,
            } => f(ErrorFields {
                message,
                code,
                status,
            }),
            #[cfg(not(feature = "axum"))]
            Self::Fail { message, code } => f(ErrorFields { message, code }),
            _ => false,
        }
    }

    #[inline]
    #[must_use]
    pub fn is_error(&self) -> bool {
        matches!(self, Self::Error { .. })
    }

    #[inline]
    #[must_use]
    pub fn is_error_and<F: FnOnce(ErrorFields<C>) -> bool>(self, f: F) -> bool {
        match self {
            #[cfg(feature = "axum")]
            Self::Error {
                message,
                code,
                status,
            } => f(ErrorFields {
                message,
                code,
                status,
            }),
            #[cfg(not(feature = "axum"))]
            Self::Error { message, code } => f(ErrorFields { message, code }),
            _ => false,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ErrorFields<C> {
    pub message: String,
    pub code: Option<C>,
    #[cfg(feature = "axum")]
    pub status: StatusCode,
}

impl<D: Serialize, E, C> From<Result<D, E>> for Brest<D, C>
where
    E: ToString,
{
    fn from(value: Result<D, E>) -> Self {
        match value {
            Ok(data) => Self::success(data),
            Err(error) => Self::error(error),
        }
    }
}

#[cfg(feature = "try")]
impl<D: Serialize, C> Try for Brest<D, C> {
    type Output = D;
    type Residual = BrestErr<C>;

    fn from_output(output: Self::Output) -> Self {
        Self::success(output)
    }

    fn branch(self) -> ControlFlow<Self::Residual, Self::Output> {
        match self {
            Brest::Success { data, .. } => ControlFlow::Continue(data),
            #[cfg(feature = "axum")]
            Brest::Error {
                message,
                code,
                status,
            } => ControlFlow::Break(BrestErr::Error {
                message,
                code,
                status,
            }),
            #[cfg(not(feature = "axum"))]
            Brest::Error { message, code } => ControlFlow::Break(BrestErr::Error { message, code }),
            #[cfg(feature = "axum")]
            Brest::Fail {
                message,
                code,
                status,
            } => ControlFlow::Break(BrestErr::Fail {
                message,
                code,
                status,
            }),
            #[cfg(not(feature = "axum"))]
            Brest::Fail { message, code } => ControlFlow::Break(BrestErr::Fail { message, code }),
        }
    }
}

#[cfg(feature = "try")]
impl<D: Serialize, C> Residual<D> for BrestErr<C> {
    type TryType = Brest<D, C>;
}

#[cfg(feature = "try")]
impl<D: Serialize, C> FromResidual<BrestErr<C>> for Brest<D, C> {
    fn from_residual(residual: BrestErr<C>) -> Self {
        residual.into()
    }
}

#[cfg(feature = "try")]
impl<D: Serialize, C, E: Into<BrestErr<C>>> FromResidual<Result<Infallible, E>> for Brest<D, C> {
    fn from_residual(residual: Result<Infallible, E>) -> Self {
        match residual {
            Err(e) => e.into().into(),
        }
    }
}

impl<D: Serialize, C, E> From<(Result<D, E>, C)> for Brest<D, C>
where
    E: ToString,
{
    fn from(value: (Result<D, E>, C)) -> Self {
        match value.0 {
            Ok(data) => Self::success(data),
            Err(error) => Self::error_code(error.to_string(), value.1),
        }
    }
}

#[cfg(feature = "axum")]
impl<D: Serialize, C, S, E> From<(Result<D, E>, C, S)> for Brest<D, C>
where
    E: ToString,
    S: Into<StatusCode>,
{
    fn from(value: (Result<D, E>, C, S)) -> Self {
        match value.0 {
            Ok(data) => Self::success(data),
            Err(error) => Self::error_code_status(error.to_string(), value.1, value.2.into()),
        }
    }
}

impl<D: Serialize, C> From<D> for Brest<D, C> {
    fn from(value: D) -> Self {
        Self::success(value)
    }
}

#[cfg(feature = "axum")]
struct BrestResponse<D: Serialize, C>(Brest<D, C>);

#[cfg(feature = "axum")]
impl<D: Serialize + 'static, C: Serialize> Serialize for BrestResponse<D, C> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut s = serializer.serialize_struct("Brest", 2)?;
        match &self.0 {
            Brest::Success { data, .. } => {
                s.serialize_field("type", "success")?;
                if std::any::TypeId::of::<D>() == std::any::TypeId::of::<()>() {
                    s.serialize_field("data", &())?;
                } else {
                    s.serialize_field("data", data)?;
                }
            }
            Brest::Error { message, code, .. } => {
                s.serialize_field("type", "error")?;
                s.serialize_field("message", message)?;
                if let Some(c) = code {
                    s.serialize_field("code", c)?;
                }
            }
            Brest::Fail { message, code, .. } => {
                s.serialize_field("type", "fail")?;
                s.serialize_field("message", message)?;
                if let Some(c) = code {
                    s.serialize_field("code", c)?;
                }
            }
        }
        s.end()
    }
}

#[cfg(feature = "axum")]
impl<D: Serialize + 'static, C: Serialize> IntoResponse for Brest<D, C> {
    fn into_response(self) -> axum::response::Response {
        use axum::Json;

        let status = match &self {
            Self::Success { status, .. } => *status,
            Self::Error { status, .. } => *status,
            Self::Fail { status, .. } => *status,
        };

        (status, Json(BrestResponse(self))).into_response()
    }
}

/// Error half of a [`Brest`], built for `?`.
///
/// Any [`std::error::Error`] converts into a `BrestErr` (status 500, `type: "error"`), so
/// `?` works on arbitrary fallible calls inside a handler returning
/// `Result<Brest<T>, BrestErr>`. Use [`ResultExt`] to change status, code or kind at the call
/// site. With the nightly `try` feature, handlers may return `Brest<T>` directly.
#[derive(Debug)]
pub enum BrestErr<C = u32> {
    Error {
        message: String,
        code: Option<C>,
        #[cfg(feature = "axum")]
        status: StatusCode,
    },
    Fail {
        message: String,
        code: Option<C>,
        #[cfg(feature = "axum")]
        status: StatusCode,
    },
}

impl<C> BrestErr<C> {
    /// An `Error` (server side problem), status 500.
    pub fn error<M: ToString>(message: M) -> Self {
        Self::Error {
            message: message.to_string(),
            code: None,
            #[cfg(feature = "axum")]
            status: StatusCode::INTERNAL_SERVER_ERROR,
        }
    }

    /// A `Fail` (caller side problem), status 400.
    pub fn fail<M: ToString>(message: M) -> Self {
        Self::Fail {
            message: message.to_string(),
            code: None,
            #[cfg(feature = "axum")]
            status: StatusCode::BAD_REQUEST,
        }
    }

    /// Sets the HTTP status.
    #[cfg(feature = "axum")]
    #[must_use]
    pub fn with_status(mut self, status: StatusCode) -> Self {
        let (Self::Error { status: s, .. } | Self::Fail { status: s, .. }) = &mut self;
        *s = status;
        self
    }

    /// Sets the application error code.
    #[must_use]
    pub fn with_code(mut self, code: C) -> Self {
        let (Self::Error { code: c, .. } | Self::Fail { code: c, .. }) = &mut self;
        *c = Some(code);
        self
    }

    /// Replaces the message.
    #[must_use]
    pub fn with_message<M: ToString>(mut self, message: M) -> Self {
        let (Self::Error { message: m, .. } | Self::Fail { message: m, .. }) = &mut self;
        *m = message.to_string();
        self
    }

    /// Turns into a `Fail`. An existing `Fail` is returned unchanged; an `Error` is rebuilt
    /// with status 400.
    #[must_use]
    pub fn into_fail(self) -> Self {
        match self {
            fail @ Self::Fail { .. } => fail,
            Self::Error { message, code, .. } => Self::Fail {
                message,
                code,
                #[cfg(feature = "axum")]
                status: StatusCode::BAD_REQUEST,
            },
        }
    }

    /// Turns into an `Error`. An existing `Error` is returned unchanged; a `Fail` is rebuilt
    /// with status 500.
    #[must_use]
    pub fn into_error(self) -> Self {
        match self {
            error @ Self::Error { .. } => error,
            Self::Fail { message, code, .. } => Self::Error {
                message,
                code,
                #[cfg(feature = "axum")]
                status: StatusCode::INTERNAL_SERVER_ERROR,
            },
        }
    }
}

// `BrestErr` deliberately does not implement `std::error::Error`; that is what makes this
// blanket impl coherent with the reflexive `From<T> for T`.
impl<C, E: std::error::Error> From<E> for BrestErr<C> {
    fn from(error: E) -> Self {
        Self::error(error)
    }
}

impl<C, T: Serialize> From<BrestErr<C>> for Brest<T, C> {
    fn from(err: BrestErr<C>) -> Self {
        match err {
            #[cfg(feature = "axum")]
            BrestErr::Error {
                message,
                code,
                status,
            } => Brest::Error {
                message,
                code,
                status,
            },
            #[cfg(not(feature = "axum"))]
            BrestErr::Error { message, code } => Brest::Error { message, code },
            #[cfg(feature = "axum")]
            BrestErr::Fail {
                message,
                code,
                status,
            } => Brest::Fail {
                message,
                code,
                status,
            },
            #[cfg(not(feature = "axum"))]
            BrestErr::Fail { message, code } => Brest::Fail { message, code },
        }
    }
}

impl<C> std::fmt::Display for BrestErr<C> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BrestErr::Error { message, .. } => write!(f, "Error: {message}"),
            BrestErr::Fail { message, .. } => write!(f, "Fail: {message}"),
        }
    }
}

#[cfg(feature = "axum")]
impl<C: Serialize> IntoResponse for BrestErr<C> {
    fn into_response(self) -> axum::response::Response {
        Brest::<(), C>::from(self).into_response()
    }
}

/// Call-site overrides for `?`; each method returns a `Result<T, BrestErr<C>>` so they chain.
///
/// `fail()` on an `Error` resets the status to 400, so call it before `status`.
/// This trait is sealed: it is implemented for every `Result<T, E>` with `E: Into<BrestErr<C>>`.
pub trait ResultExt<T, C>: sealed::Sealed<T, C> {
    /// Sets the HTTP status of the resulting error.
    #[cfg(feature = "axum")]
    fn status(self, status: StatusCode) -> Result<T, BrestErr<C>>;
    /// Sets the application error code.
    fn code(self, code: C) -> Result<T, BrestErr<C>>;
    /// Marks the error as a `Fail` (status 400 unless it already was a `Fail`).
    fn fail(self) -> Result<T, BrestErr<C>>;
    /// Replaces the error message.
    fn message<M: ToString>(self, message: M) -> Result<T, BrestErr<C>>;
}

mod sealed {
    pub trait Sealed<T, C> {}
    impl<T, E: Into<super::BrestErr<C>>, C> Sealed<T, C> for Result<T, E> {}
}

impl<T, E: Into<BrestErr<C>>, C> ResultExt<T, C> for Result<T, E> {
    #[cfg(feature = "axum")]
    fn status(self, status: StatusCode) -> Result<T, BrestErr<C>> {
        self.map_err(|e| e.into().with_status(status))
    }

    fn code(self, code: C) -> Result<T, BrestErr<C>> {
        self.map_err(|e| e.into().with_code(code))
    }

    fn fail(self) -> Result<T, BrestErr<C>> {
        self.map_err(|e| e.into().into_fail())
    }

    fn message<M: ToString>(self, message: M) -> Result<T, BrestErr<C>> {
        self.map_err(|e| e.into().with_message(message))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_success_construction() {
        let brest = Brest::<String, u32>::success("test data".to_string());
        assert!(brest.is_success());
        assert!(!brest.is_error());
        assert!(!brest.is_fail());
    }

    #[test]
    fn test_error_construction() {
        let brest = Brest::<(), u32>::error("error message");
        assert!(!brest.is_success());
        assert!(brest.is_error());
        assert!(!brest.is_fail());
    }

    #[test]
    fn test_error_code_construction() {
        let brest = Brest::<(), u32>::error_code("error message", 404);
        assert!(!brest.is_success());
        assert!(brest.is_error());
        assert!(!brest.is_fail());
    }

    #[test]
    fn test_fail_construction() {
        let brest = Brest::<(), u32>::fail("fail message");
        assert!(!brest.is_success());
        assert!(!brest.is_error());
        assert!(brest.is_fail());
    }

    #[test]
    fn test_fail_code_construction() {
        let brest = Brest::<(), u32>::fail_code("fail message", 400);
        assert!(!brest.is_success());
        assert!(!brest.is_error());
        assert!(brest.is_fail());
    }

    #[test]
    fn test_success_serialization() {
        let brest = Brest::<String, u32>::success("test data".to_string());
        let json = serde_json::to_string(&brest).unwrap();
        assert_eq!(json, r#"{"type":"success","data":"test data"}"#);
    }

    #[test]
    fn test_success_unit_serialization() {
        let brest = Brest::<(), u32>::success(());
        let json = serde_json::to_string(&brest).unwrap();
        assert_eq!(json, r#"{"type":"success","data":null}"#);
    }

    #[test]
    fn test_error_serialization() {
        let brest = Brest::<(), u32>::error("error message");
        let json = serde_json::to_string(&brest).unwrap();
        assert_eq!(json, r#"{"type":"error","message":"error message"}"#);
    }

    #[test]
    fn test_error_code_serialization() {
        let brest = Brest::<(), u32>::error_code("error message", 500);
        let json = serde_json::to_string(&brest).unwrap();
        assert_eq!(
            json,
            r#"{"type":"error","message":"error message","code":500}"#
        );
    }

    #[test]
    fn test_fail_serialization() {
        let brest = Brest::<(), u32>::fail("fail message");
        let json = serde_json::to_string(&brest).unwrap();
        assert_eq!(json, r#"{"type":"fail","message":"fail message"}"#);
    }

    #[test]
    fn test_fail_code_serialization() {
        let brest = Brest::<(), u32>::fail_code("fail message", 400);
        let json = serde_json::to_string(&brest).unwrap();
        assert_eq!(
            json,
            r#"{"type":"fail","message":"fail message","code":400}"#
        );
    }

    #[test]
    fn test_success_deserialization() {
        let json = r#"{"type":"success","data":"test data"}"#;
        let brest: Brest<String, u32> = serde_json::from_str(json).unwrap();
        assert!(brest.is_success());
    }

    #[test]
    fn test_error_deserialization() {
        let json = r#"{"type":"error","message":"error message"}"#;
        let brest: Brest<(), u32> = serde_json::from_str(json).unwrap();
        assert!(brest.is_error());
    }

    #[test]
    fn test_error_code_deserialization() {
        let json = r#"{"type":"error","message":"error message","code":500}"#;
        let brest: Brest<(), u32> = serde_json::from_str(json).unwrap();
        assert!(brest.is_error());
    }

    #[test]
    fn test_fail_deserialization() {
        let json = r#"{"type":"fail","message":"fail message"}"#;
        let brest: Brest<(), u32> = serde_json::from_str(json).unwrap();
        assert!(brest.is_fail());
    }

    #[test]
    fn test_fail_code_deserialization() {
        let json = r#"{"type":"fail","message":"fail message","code":400}"#;
        let brest: Brest<(), u32> = serde_json::from_str(json).unwrap();
        assert!(brest.is_fail());
    }

    #[test]
    fn test_is_success_and() {
        let brest = Brest::<i32, u32>::success(42);
        assert!(brest.is_success_and(|x| x == 42));

        let brest2 = Brest::<i32, u32>::success(42);
        assert!(!brest2.is_success_and(|x| x == 0));
    }

    #[test]
    fn test_is_error_and() {
        let brest = Brest::<(), u32>::error_code("test", 500);
        assert!(brest.is_error_and(|fields| fields.message == "test" && fields.code == Some(500)));

        let brest2 = Brest::<(), u32>::error_code("test", 500);
        assert!(!brest2.is_error_and(|fields| fields.code == Some(400)));
    }

    #[test]
    fn test_is_fail_and() {
        let brest = Brest::<(), u32>::fail_code("test", 400);
        assert!(brest.is_fail_and(|fields| fields.message == "test" && fields.code == Some(400)));

        let brest2 = Brest::<(), u32>::fail_code("test", 400);
        assert!(!brest2.is_fail_and(|fields| fields.code == Some(500)));
    }

    #[test]
    fn test_from_result_ok() {
        let result: Result<String, String> = Ok("success".to_string());
        let brest: Brest<String, u32> = result.into();
        assert!(brest.is_success());
    }

    #[test]
    fn test_from_result_err() {
        let result: Result<String, String> = Err("error".to_string());
        let brest: Brest<String, u32> = result.into();
        assert!(brest.is_error());
    }

    #[test]
    fn test_from_result_with_code_ok() {
        let result: (Result<String, String>, u32) = (Ok("success".to_string()), 200);
        let brest: Brest<String, u32> = result.into();
        assert!(brest.is_success());
    }

    #[test]
    fn test_from_result_with_code_err() {
        let result: (Result<String, String>, u32) = (Err("error".to_string()), 500);
        let brest: Brest<String, u32> = result.into();
        assert!(brest.is_error());
    }

    #[test]
    fn test_from_data() {
        let data = "test".to_string();
        let brest: Brest<String, u32> = data.into();
        assert!(brest.is_success());
    }

    #[cfg(feature = "try")]
    mod try_tests {
        use super::*;

        #[test]
        fn test_try_success() {
            let brest = Brest::<i32, u32>::success(42);
            let result: ControlFlow<BrestErr<u32>, i32> = brest.branch();
            match result {
                ControlFlow::Continue(val) => assert_eq!(val, 42),
                _ => panic!("Expected Continue"),
            }
        }

        #[test]
        fn test_try_error() {
            let brest = Brest::<i32, u32>::error("test error");
            let result: ControlFlow<BrestErr<u32>, i32> = brest.branch();
            assert!(result.is_break());
        }

        #[test]
        fn test_try_fail() {
            let brest = Brest::<i32, u32>::fail("test fail");
            let result: ControlFlow<BrestErr<u32>, i32> = brest.branch();
            assert!(result.is_break());
        }

        #[test]
        fn test_from_residual() {
            let residual = BrestErr::<u32>::error("residual");
            let brest: Brest<i32, u32> = Brest::from_residual(residual);
            assert!(brest.is_error());
        }

        #[test]
        fn test_from_result_residual() {
            let residual: Result<Infallible, BrestErr<u32>> = Err(BrestErr::fail("residual"));
            let brest: Brest<i32, u32> = Brest::from_residual(residual);
            assert!(brest.is_fail());
        }
    }

    #[cfg(feature = "axum")]
    mod axum_tests {
        use super::*;
        use axum::http::StatusCode;

        #[test]
        fn test_success_status() {
            let brest =
                Brest::<String, u32>::success_status("data".to_string(), StatusCode::CREATED);
            assert!(brest.is_success());
        }

        #[test]
        fn test_error_status() {
            let brest = Brest::<(), u32>::error_status("error", StatusCode::NOT_FOUND);
            assert!(brest.is_error());
        }

        #[test]
        fn test_error_code_status() {
            let brest = Brest::<(), u32>::error_code_status("error", 404, StatusCode::NOT_FOUND);
            assert!(brest.is_error());
        }

        #[test]
        fn test_fail_status() {
            let brest = Brest::<(), u32>::fail_status("fail", StatusCode::UNAUTHORIZED);
            assert!(brest.is_fail());
        }

        #[test]
        fn test_fail_code_status() {
            let brest = Brest::<(), u32>::fail_code_status("fail", 401, StatusCode::UNAUTHORIZED);
            assert!(brest.is_fail());
        }

        #[test]
        fn test_from_result_with_status_ok() {
            let result: (Result<String, String>, u32, StatusCode) =
                (Ok("success".to_string()), 200, StatusCode::OK);
            let brest: Brest<String, u32> = result.into();
            assert!(brest.is_success());
        }

        #[test]
        fn test_from_result_with_status_err() {
            let result: (Result<String, String>, u32, StatusCode) = (
                Err("error".to_string()),
                500,
                StatusCode::INTERNAL_SERVER_ERROR,
            );
            let brest: Brest<String, u32> = result.into();
            assert!(brest.is_error());
        }

        #[test]
        fn test_brest_response_success_serialization() {
            let brest = Brest::<String, u32>::success("test".to_string());
            let response = BrestResponse(brest);
            let json = serde_json::to_string(&response).unwrap();
            assert_eq!(json, r#"{"type":"success","data":"test"}"#);
        }

        #[test]
        fn test_brest_response_success_unit_serialization() {
            let brest = Brest::<(), u32>::success(());
            let response = BrestResponse(brest);
            let json = serde_json::to_string(&response).unwrap();
            assert_eq!(json, r#"{"type":"success","data":null}"#);
        }

        #[test]
        fn test_brest_response_error_serialization() {
            let brest = Brest::<(), u32>::error_code("error", 500);
            let response = BrestResponse(brest);
            let json = serde_json::to_string(&response).unwrap();
            assert_eq!(json, r#"{"type":"error","message":"error","code":500}"#);
        }

        #[test]
        fn test_brest_response_fail_serialization() {
            let brest = Brest::<(), u32>::fail_code("fail", 400);
            let response = BrestResponse(brest);
            let json = serde_json::to_string(&response).unwrap();
            assert_eq!(json, r#"{"type":"fail","message":"fail","code":400}"#);
        }

        #[test]
        fn test_into_response() {
            let brest = Brest::<String, u32>::success("test".to_string());
            let response = brest.into_response();
            assert_eq!(response.status(), StatusCode::OK);
        }

        #[test]
        fn test_brest_err_from() {
            let err = BrestErr::Error {
                message: "test".to_string(),
                code: Some(500),
                status: StatusCode::INTERNAL_SERVER_ERROR,
            };
            let brest: Brest<(), u32> = err.into();
            assert!(brest.is_error());
        }
    }

    #[cfg(feature = "schemars")]
    mod schemars_tests {
        use super::*;

        #[test]
        fn test_json_schema() {
            let schema =
                schemars::SchemaGenerator::default().into_root_schema_for::<Brest<String, u32>>();
            assert!(schema.get("oneOf").is_some());
        }
    }
}
