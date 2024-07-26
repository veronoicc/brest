use std::fmt::{Debug, Display};

use serde::{Deserialize, Serialize};

#[cfg(feature = "schemars")]
use schemars::JsonSchema;

#[derive(Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "schemars", derive(JsonSchema))]
#[serde(rename_all = "lowercase", rename_all_fields = "lowercase")]
#[serde(tag = "type")]
pub enum Brest<D, C = u32>
{
    Success(D),
    Error {
        message: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        code: Option<C>,
    },
    Fail {
        message: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        code: Option<C>,
    },
}

impl<D, C> Brest<D, C>
{
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

impl<D, C> Brest<D, C>
{
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
pub struct ErrorFields<C>
{
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

impl<D, C, E> From<(Result<D, E>, C)> for Brest<D, C>
where
    E: Display,
{
    fn from(value: (Result<D, E>, C)) -> Self {
        match value.0 {
            Ok(data) => Self::success(data),
            Err(error) => Self::error_code(error.to_string(), value.1),
        }
    }
}
