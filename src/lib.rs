#![cfg_attr(feature = "try", feature(try_trait_v2))]

#[cfg(feature = "try")]
use std::ops::{ControlFlow, FromResidual, Try};
#[cfg(not(feature = "try"))]
#[cfg(feature = "try")]
use std::ops::FromResidual;

#[cfg(feature = "axum")]
use axum::{http::StatusCode, response::IntoResponse};
#[cfg(feature = "serde")]
use serde::{
    ser::{SerializeStruct, Serializer},
    Deserialize, Serialize,
};

#[cfg(feature = "schemars")]
use schemars::JsonSchema;


use std::fmt::Debug;

#[derive(Debug)]
#[cfg_attr(feature = "schemars", derive(JsonSchema))]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize), serde(rename_all = "lowercase", rename_all_fields = "lowercase", tag = "type", content = "data"))]
pub enum Brest<D = (), C = u32> {
    Success(D, #[cfg(feature = "axum")] #[serde(skip)] StatusCode),
    Error {
        message: String,
        #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
        code: Option<C>,
        #[cfg(feature = "axum")]
        #[serde(skip)]
        status: StatusCode,
    },
    Fail {
        message: String,
        #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
        code: Option<C>,
        #[cfg(feature = "axum")]
        #[serde(skip)]
        status: StatusCode,
    },
}

impl<D, C> Brest<D, C> {
    pub fn success(data: D) -> Self {
        Self::Success(data, #[cfg(feature = "axum")] StatusCode::OK)
    }

    #[cfg(feature = "axum")]
    pub fn success_status(data: D, status: StatusCode) -> Self {
        Self::Success(data, status)
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
            #[cfg(feature = "axum")]
            Self::Success(data, _) => f(data),
            #[cfg(not(feature = "axum"))]
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
            #[cfg(feature = "axum")]
            Self::Fail { message, code, status } => f(ErrorFields { message, code, status }),
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
            Self::Error { message, code, status } => f(ErrorFields { message, code, status }),
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
impl<D, E, C> FromResidual<Result<D, E>> for Brest<D, C>
where
    E: ToString,
{
    fn from_residual(residual: Result<D, E>) -> Self {
        Self::error(residual.err().unwrap().to_string())
    }
}

#[cfg(feature = "try")]
impl<D, C> Try for Brest<D, C> {
    type Output = D;
    type Residual = Brest<(), C>;

    fn from_output(output: Self::Output) -> Self {
        Self::success(output)
    }

    fn branch(self) -> ControlFlow<Self::Residual, Self::Output> {
        match self {
            #[cfg(feature = "axum")]
            Brest::Success(data, _) => ControlFlow::Continue(data),
            #[cfg(not(feature = "axum"))]
            Brest::Success(data) => ControlFlow::Continue(data),
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
impl<D, C> FromResidual<Brest<(), C>> for Brest<D, C> {
    fn from_residual(residual: Brest<(), C>) -> Self {
        match residual {
            #[cfg(feature = "axum")]
            Brest::Success(_, _) => unreachable!(),
            #[cfg(not(feature = "axum"))]
            Brest::Success(_) => unreachable!(),
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
impl<D, E, C> FromResidual<(Result<D, E>, C)> for Brest<D, C>
where
    E: ToString,
{
    fn from_residual(residual: (Result<D, E>, C)) -> Self {
        Self::error_code(residual.0.err().unwrap().to_string(), residual.1)
    }
}

#[cfg(feature = "axum")]
impl<D, C, S, E> From<(Result<D, E>, C, S)> for Brest<D, C>
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
impl<D, E, S, C> FromResidual<(Result<D, E>, C, S)> for Brest<D, C>
where
    E: ToString,
    S: Into<StatusCode>,
{
    fn from_residual(residual: (Result<D, E>, C, S)) -> Self {
        Self::error_code_status(residual.0.err().unwrap().to_string(), residual.1, residual.2.into())
    }
}

impl<D, C> From<D> for Brest<D, C> {
    fn from(value: D) -> Self {
        Self::success(value)
    }
}

#[cfg(all(feature = "axum", feature = "serde"))]
struct BrestResponse<D, C>(Brest<D, C>);

#[cfg(all(feature = "axum", feature = "serde"))]
impl<D, C> Serialize for BrestResponse<D, C>
where
    D: Serialize + 'static,
    C: Serialize,
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match &self.0 {
            Brest::Success(data, _) if std::any::TypeId::of::<D>() == std::any::TypeId::of::<()>() => {
                let mut s = serializer.serialize_struct("Brest", 2)?;
                s.serialize_field("type", "success")?;
                s.serialize_field("data", &())?;
                s.end()
            }
            _ => self.0.serialize(serializer),
        }
    }
}

#[cfg(all(feature = "axum", feature = "serde"))]
impl<D, C> IntoResponse for Brest<D, C>
where
    D: Serialize + 'static,
    C: Serialize,
{
    fn into_response(self) -> axum::response::Response {
        use axum::Json;

        let status = match &self {
            Self::Success(_, status) => *status,
            Self::Error { status, .. } => *status,
            Self::Fail { status, .. } => *status,
        };

        (status, Json(BrestResponse(self))).into_response()
    }
}

#[cfg(feature = "try")]
impl<D, C, U> FromResidual<Result<U, Self>> for Brest<D, C>
{
    fn from_residual(residual: Result<U, Self>) -> Self {
        residual.err().unwrap()
    }
}