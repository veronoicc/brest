#![cfg_attr(feature = "try", feature(try_trait_v2))]

#[cfg(feature = "try")]
use std::ops::{ControlFlow, FromResidual, Try};

#[cfg(feature = "axum")]
use axum::{http::StatusCode, response::IntoResponse};

#[cfg(feature = "schemars")]
use schemars::JsonSchema;

use serde::{Deserialize, Serialize, Serializer, ser::SerializeStruct as _};

use std::fmt::Debug;

#[derive(Debug)]
#[cfg_attr(feature = "schemars", derive(JsonSchema))]
#[derive(Serialize, Deserialize)]
#[serde(
    rename_all = "lowercase",
    rename_all_fields = "lowercase",
    tag = "type",
    content = "data"
)]
pub enum Brest<D: Serialize = (), C = u32> {
    Success {
        #[serde(flatten)]
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
impl<D: Serialize, E, C> FromResidual<Result<D, E>> for Brest<D, C>
where
    E: ToString,
{
    fn from_residual(residual: Result<D, E>) -> Self {
        Self::error(residual.err().unwrap().to_string())
    }
}

#[cfg(feature = "try")]
impl<D: Serialize, C> Try for Brest<D, C> {
    type Output = D;
    type Residual = Brest<(), C>;

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
            } => ControlFlow::Break(Brest::Error {
                message,
                code,
                status,
            }),
            #[cfg(not(feature = "axum"))]
            Brest::Error { message, code } => ControlFlow::Break(Brest::Error { message, code }),
            #[cfg(feature = "axum")]
            Brest::Fail {
                message,
                code,
                status,
            } => ControlFlow::Break(Brest::Fail {
                message,
                code,
                status,
            }),
            #[cfg(not(feature = "axum"))]
            Brest::Fail { message, code } => ControlFlow::Break(Brest::Fail { message, code }),
        }
    }
}

#[cfg(feature = "try")]
impl<D: Serialize, C> FromResidual<Brest<(), C>> for Brest<D, C> {
    fn from_residual(residual: Brest<(), C>) -> Self {
        match residual {
            Brest::Success { .. } => unreachable!(),
            #[cfg(feature = "axum")]
            Brest::Error {
                message,
                code,
                status,
            } => Brest::Error {
                message,
                code,
                status,
            },
            #[cfg(not(feature = "axum"))]
            Brest::Error { message, code } => Brest::Error { message, code },
            #[cfg(feature = "axum")]
            Brest::Fail {
                message,
                code,
                status,
            } => Brest::Fail {
                message,
                code,
                status,
            },
            #[cfg(not(feature = "axum"))]
            Brest::Fail { message, code } => Brest::Fail { message, code },
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

#[cfg(feature = "try")]
impl<D: Serialize, E, C> FromResidual<(Result<D, E>, C)> for Brest<D, C>
where
    E: ToString,
{
    fn from_residual(residual: (Result<D, E>, C)) -> Self {
        Self::error_code(residual.0.err().unwrap().to_string(), residual.1)
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

#[cfg(all(feature = "try", feature = "axum"))]
impl<D: Serialize, E, S, C> FromResidual<(Result<D, E>, C, S)> for Brest<D, C>
where
    E: ToString,
    S: Into<StatusCode>,
{
    fn from_residual(residual: (Result<D, E>, C, S)) -> Self {
        Self::error_code_status(
            residual.0.err().unwrap().to_string(),
            residual.1,
            residual.2.into(),
        )
    }
}

impl<D: Serialize, C> From<D> for Brest<D, C> {
    fn from(value: D) -> Self {
        Self::success(value)
    }
}

#[cfg(all(feature = "axum"))]
struct BrestResponse<D: Serialize, C>(Brest<D, C>);

#[cfg(all(feature = "axum"))]
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

#[cfg(all(feature = "axum"))]
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

#[cfg(feature = "axum")]
#[derive(Debug)]
pub enum BrestErr<C = u32> {
    Error {
        message: String,
        code: Option<C>,
        status: StatusCode,
    },
    Fail {
        message: String,
        code: Option<C>,
        status: StatusCode,
    },
}

#[cfg(feature = "axum")]
impl<C, T: Serialize> From<BrestErr<C>> for Brest<T, C> {
    fn from(err: BrestErr<C>) -> Self {
        match err {
            BrestErr::Error { message, code, status } => Brest::Error { message, code, status },
            BrestErr::Fail { message, code, status } => Brest::Fail { message, code, status },
        }
    }
}

#[cfg(feature = "axum")]
impl<C: std::fmt::Debug> std::error::Error for BrestErr<C> {}

#[cfg(feature = "axum")]
impl<C: std::fmt::Debug> std::fmt::Display for BrestErr<C> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BrestErr::Error { message, .. } => write!(f, "Error: {}", message),
            BrestErr::Fail { message, .. } => write!(f, "Fail: {}", message),
        }
    }
}

#[cfg(feature = "try")]
impl<D: Serialize, C, U> FromResidual<Result<U, Self>> for Brest<D, C> {
    fn from_residual(residual: Result<U, Self>) -> Self {
        residual.err().unwrap()
    }
}
