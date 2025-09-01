use std::ops::Deref;

use axum::extract::{FromRequest, Request};
use axum::extract::rejection::{ExtensionRejection, FormRejection, JsonRejection, PathRejection, QueryRejection};
use axum::response::IntoResponse;
use serde::Serialize;

use crate::Brest;

#[derive(FromRequest)]
#[from_request(via(axum::Json), rejection(Brest))]
pub struct Json<T>(pub T);

impl<T: Serialize> IntoResponse for Json<T> {
    fn into_response(self) -> axum::response::Response {
        let Self(value) = self;
        axum::Json(value).into_response()
    }
}

impl From<JsonRejection> for Brest {
    fn from(value: JsonRejection) -> Self {
        Brest::fail_status(value.body_text(), value.status())
    }
}

impl<T> Deref for Json<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

pub struct Bytes(pub axum::body::Bytes);

impl<S> axum::extract::FromRequest<S> for Bytes
where
    S: Send + Sync,
{
    type Rejection = Brest;

    async fn from_request(req: Request, state: &S) -> Result<Self, Self::Rejection> {
        match axum::body::Bytes::from_request(req, state).await {
            Ok(b) => Ok(Bytes(b)),
            Err(e) => Err(Brest::fail_status(e.body_text(), e.status())),
        }
    }
}

impl IntoResponse for Bytes
{
    fn into_response(self) -> axum::response::Response {
        let Self(value) = self;
        value.into_response()
    }
}

impl Deref for Bytes {
    type Target = axum::body::Bytes;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(FromRequest)]
#[from_request(via(axum::Extension), rejection(Brest))]
pub struct Extension<T>(pub T);

impl<T> IntoResponse for Extension<T>
where
    T: Clone + Send + Sync + 'static,
{
    fn into_response(self) -> axum::response::Response {
        let Self(value) = self;
        axum::Extension(value).into_response()
    }
}

impl From<ExtensionRejection> for Brest {
    fn from(value: ExtensionRejection) -> Self {
        Brest::fail_status(value.body_text(), value.status())
    }
}

impl<T> Deref for Extension<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(FromRequest)]
#[from_request(via(axum::Form), rejection(Brest))]
pub struct Form<T>(pub T);

impl<T: Serialize> IntoResponse for Form<T> {
    fn into_response(self) -> axum::response::Response {
        let Self(value) = self;
        axum::Form(value).into_response()
    }
}

impl From<FormRejection> for Brest {
    fn from(value: FormRejection) -> Self {
        Brest::fail_status(value.body_text(), value.status())
    }
}

impl<T> Deref for Form<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

pub struct MatchedPath(axum::extract::MatchedPath);

impl<S> axum::extract::FromRequestParts<S> for MatchedPath
where
    S: Send + Sync,
{
    type Rejection = Brest;

    async fn from_request_parts(
        parts: &mut axum::http::request::Parts,
        _state: &S,
    ) -> Result<Self, Self::Rejection> {
        match axum::extract::MatchedPath::from_request_parts(parts, _state).await {
            Ok(mp) => Ok(MatchedPath(mp)),
            Err(e) => Err(Brest::fail_status(e.body_text(), e.status())),
        }
    }
}

impl Deref for MatchedPath {
    type Target = axum::extract::MatchedPath;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(FromRequest)]
#[from_request(via(axum::extract::Path), rejection(Brest))]
pub struct Path<T>(pub T);

impl From<PathRejection> for Brest {
    fn from(value: PathRejection) -> Self {
        Brest::fail_status(value.body_text(), value.status())
    }
}

impl<T> Deref for Path<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(FromRequest)]
#[from_request(via(axum::extract::Query), rejection(Brest))]
pub struct Query<T>(pub T);

impl From<QueryRejection> for Brest {
    fn from(value: QueryRejection) -> Self {
        Brest::fail_status(value.body_text(), value.status())
    }
}

impl<T> Deref for Query<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

pub struct RawForm(axum::extract::RawForm);

impl<S> axum::extract::FromRequest<S> for RawForm
where
    S: Send + Sync,
{
    type Rejection = Brest;

    async fn from_request(req: Request, state: &S) -> Result<Self, Self::Rejection> {
        match axum::extract::RawForm::from_request(req, state).await {
            Ok(rf) => Ok(RawForm(rf)),
            Err(e) => Err(Brest::fail_status(e.body_text(), e.status())),
        }
    }
}

impl Deref for RawForm {
    type Target = axum::extract::RawForm;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

pub struct RawPathParams(axum::extract::RawPathParams);

impl<S> axum::extract::FromRequest<S> for RawPathParams
where
    S: Send + Sync,
{
    type Rejection = Brest;

    async fn from_request(req: Request, state: &S) -> Result<Self, Self::Rejection> {
        match axum::extract::RawPathParams::from_request(req, state).await {
            Ok(rpp) => Ok(RawPathParams(rpp)),
            Err(e) => Err(Brest::fail_status(e.body_text(), e.status())),
        }
    }
}

impl Deref for RawPathParams {
    type Target = axum::extract::RawPathParams;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}