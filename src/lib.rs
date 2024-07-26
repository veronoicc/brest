#![cfg_attr(feature = "try", feature(try_trait_v2))]

#[cfg(feature = "try")]
use std::{convert::Infallible, ops::FromResidual};

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

#[cfg(feature = "schemars")]
use schemars::JsonSchema;

use std::fmt::Debug;

#[derive(Debug)]
#[cfg_attr(feature = "schemars", derive(JsonSchema))]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize), serde(rename_all = "lowercase", rename_all_fields = "lowercase", tag = "type"))]
pub enum Brest<D = (), C = u32> {
    Success(D),
    Error {
        message: String,
        #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
        code: Option<C>,
    },
    Fail {
        message: String,
        #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
        code: Option<C>,
    },
}

impl<D, C> Brest<D, C> {
    pub fn success(data: D) -> Self {
        Self::Success(data)
    }

    pub fn error<M: ToString>(message: M) -> Self {
        Self::Error {
            message: message.to_string(),
            code: None,
        }
    }

    pub fn error_code<M: ToString>(message: M, code: C) -> Self {
        Self::Error {
            message: message.to_string(),
            code: Some(code),
        }
    }

    pub fn fail<M: ToString>(message: M) -> Self {
        Self::Fail {
            message: message.to_string(),
            code: None,
        }
    }

    pub fn fail_code<M: ToString>(message: M, code: C) -> Self {
        Self::Fail {
            message: message.to_string(),
            code: Some(code),
        }
    }
}

impl<D, C> Brest<D, C> {
    #[inline]
    #[must_use]
    pub fn is_success(&self) -> bool {
        matches!(self, Self::Success { .. })
    }

    #[inline]
    #[must_use]
    pub fn is_success_and<F: FnOnce(D) -> bool>(self, f: F) -> bool {
        match self {
            Self::Success(data) => f(data),
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
            Self::Error { message, code } => f(ErrorFields { message, code }),
            _ => false,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ErrorFields<C> {
    pub message: String,
    pub code: Option<C>,
}

impl<D, E, C> From<Result<D, E>> for Brest<D, C>
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
impl<D, E, C> FromResidual<Result<Infallible, E>> for Brest<D, C>
where
    E: ToString,
{
    fn from_residual(residual: Result<Infallible, E>) -> Self {
        Self::error(residual.err().unwrap().to_string())
    }
}

impl<D, C, E> From<(Result<D, E>, C)> for Brest<D, C>
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

#[cfg(feature = "try")]
impl<D, E, C> FromResidual<(Result<Infallible, E>, C)> for Brest<D, C>
where
    E: ToString,
{
    fn from_residual(residual: (Result<Infallible, E>, C)) -> Self {
        Self::error_code(residual.0.err().unwrap().to_string(), residual.1)
    }
}

impl<D, C> From<D> for Brest<D, C> {
    fn from(value: D) -> Self {
        Self::success(value)
    }
}