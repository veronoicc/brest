use std::fmt::{self, Debug, Display};

use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "lowercase", rename_all_fields = "lowercase")]
pub enum Brest<D, C = u32> where D: Serialize, C: num_traits::PrimInt + Serialize {
    Success {
        data: D
    },
    Error {
        message: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        code: Option<C>,
    },
    Fail {
        message: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        code: Option<C>,
    }
}

impl<D, C> Brest<D, C> where D: Serialize, C: num_traits::PrimInt + Serialize {
    pub fn success(data: D) -> Self {
        Self::Success {
            data
        }
    }

    pub fn error<M: ToString>(message: M) -> Self {
        Self::Error {
            message: message.to_string(),
            code: None
        }
    }
    
    pub fn error_code<M: ToString>(message: M, code: C) -> Self {
        Self::Error {
            message: message.to_string(),
            code: Some(code)
        }
    }

    pub fn fail<M: ToString>(message: M) -> Self {
        Self::Fail {
            message: message.to_string(),
            code: None
        }
    }

    pub fn fail_code<M: ToString>(message: M, code: C) -> Self {
        Self::Fail {
            message: message.to_string(),
            code: Some(code)
        }
    }
}

impl<D, C> Brest<D, C> where D: Serialize + Debug, C: num_traits::PrimInt + Serialize + Debug {
    #[inline]
    #[track_caller]
    pub fn unwrap(self) -> D
    {
        match self {
            Self::Success { data } => data,
            Self::Error {
                message,
                code,
            } => unwrap_failed(
                "called `RJSend::unwrap()` on an `Error` value",
                &ErrorFields {
                    message,
                    code,
                },
            ),
            Self::Fail {
                message,
                code
            } => {
                unwrap_failed(
                    "called `RJSend::unwrap()` on a `Fail` value",
                    &ErrorFields {
                        message,
                        code,
                    })
            },
        }
    }

    #[inline]
    #[track_caller]
    pub fn unwrap_error(self) -> ErrorFields<C>
    {
        match self {
            Self::Error {
                message,
                code,
            } => ErrorFields {
                message,
                code,
            },
            Self::Success {
                data
            } => unwrap_failed(
                "called `RJSend::unwrap_error()` on an `Success` value",
                &data,
            ),
            Self::Fail {
                message,
                code
            } => {
                unwrap_failed(
                    "called `RJSend::unwrap_error()` on a `Fail` value",
                    &ErrorFields {
                        message,
                        code,
                    })
            },
        }
    }

    #[inline]
    #[track_caller]
    pub fn unwrap_fail(self) -> ErrorFields<C>
    {
        match self {
            Self::Fail {
                message,
                code,
            } => ErrorFields {
                message,
                code,
            },
            Self::Error {
                message,
                code
            } => {
                unwrap_failed(
                    "called `RJSend::unwrap_fail()` on a `Error` value",
                    &ErrorFields {
                        message,
                        code,
                    })
            },
            Self::Success {
                data
            } => unwrap_failed(
                "called `RJSend::unwrap_fail()` on an `Success` value",
                &data,
            ),
        }
    }

    #[inline]
    pub fn unwrap_or(self, default: D) -> D {
        match self {
            Self::Success { data } => data,
            _ => default,
        }
    }

    #[inline]
    pub fn unwrap_or_else<F>(self, f: F) -> D
    where
        F: FnOnce() -> D,
    {
        match self {
            Self::Success { data } => data,
            _ => f(),
        }
    }

    #[inline]
    #[allow(renamed_and_removed_lints)]
    #[allow(clippy::unwrap_or_else_default)]
    pub fn unwrap_or_default(self) -> D
    where
        D: Default,
    {
        // NOTE: We need to add a linter exception here,
        // because we are *not* using `std::option::Option`,
        // or `std::result::Result` here,
        // and actually *do* want to use `RJSend::unwrap_or_else` here,
        // because we're implementing `RJSend::unwrap_or_default` here... xD
        //
        // Also, `unwrap_or_else_default` was quite recently renamed,
        // making using the old name, and adding an exception to allow it,
        // the easiest solution, whilst retaining the current implementation...
        self.unwrap_or_else(Default::default)
    }
}

impl<D, C> Brest<D, C> where D: Serialize + Debug, C: num_traits::PrimInt + Serialize + Debug {
    #[inline]
    #[track_caller]
    pub fn expect(self, msg: &str) -> D
    {
        match self {
            Self::Success { data } => data,
            Self::Fail {
                message,
                code,
            } => unwrap_failed(
                msg,
                &ErrorFields {
                    message,
                    code,
                },
            ),
            Self::Error {
                message,
                code,
            } => unwrap_failed(
                msg,
                &ErrorFields {
                    message,
                    code,
                },
            ),
        }
    }

    #[inline]
    #[track_caller]
    pub fn expect_error(self, msg: &str) -> ErrorFields<C>
    {
        match self {
            Self::Error {
                message,
                code,
            } => ErrorFields {
                message,
                code,
            },
            Self::Success { data } => unwrap_failed(
                msg,
                &data,
            ),
            Self::Fail {
                message,
                code,
            } => unwrap_failed(
                msg,
                &ErrorFields {
                    message,
                    code,
                },
            ),
        }
    }

    #[inline]
    #[track_caller]
    pub fn expect_fail(self, msg: &str) -> ErrorFields<C>
    {
        match self {
            Self::Fail {
                message,
                code,
            } => ErrorFields {
                message,
                code,
            },
            Self::Error {
                message,
                code,
            } => unwrap_failed(
                msg,
                &ErrorFields {
                    message,
                    code,
                },
            ),
            Self::Success { data } => unwrap_failed(
                msg,
                &data,
            ),
        }
    }
}

#[inline(never)]
#[cold]
#[track_caller]
fn unwrap_failed(msg: &str, error: &dyn fmt::Debug) -> ! {
    panic!("{}: {:?}", msg, error)
}

impl<D, C> Brest<D, C> where D: Serialize, C: num_traits::PrimInt + Serialize {
    #[inline]
    #[must_use]
    pub fn is_success(&self) -> bool {
        matches!(self, Self::Success { .. })
    }

    #[inline]
    #[must_use]
    pub fn is_success_and<F: FnOnce(D) -> bool>(self, f: F) -> bool {
        match self {
            Self::Success { data } => f(data),
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
            Self::Fail { 
                message,
                code
            } => f(ErrorFields {
                message,
                code,
            }),
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
            Self::Error {
                message,
                code,
            } => f(ErrorFields {
                message,
                code,
            }),
            _ => false,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ErrorFields<C> where C: num_traits::PrimInt + Serialize {
    pub message: String,
    pub code: Option<C>,
}

impl<D, C, E> From<Result<D, E>> for Brest<D, C> where D: Serialize, C: num_traits::PrimInt + Serialize, E: Display {
    fn from(value: Result<D, E>) -> Self {
        match value {
            Ok(data) => Self::success(data),
            Err(error) => Self::error(error.to_string()),
        }
    }
}

impl<D, C, E> From<(Result<D, E>, C)> for Brest<D, C> where D: Serialize, C: num_traits::PrimInt + Serialize, E: Display {
    fn from(value: (Result<D, E>, C)) -> Self {
        match value.0 {
            Ok(data) => Self::success(data),
            Err(error) => Self::error_code(error.to_string(), value.1),
        }
    }
}