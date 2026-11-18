#![cfg(feature = "axum")]

use axum::{
    Router,
    body::Body,
    http::{Request, StatusCode},
    routing::get,
};
use brest::{Brest, BrestErr, ResultExt as _};
use http_body_util::BodyExt as _;
use serde_json::{Value, json};
use tower::ServiceExt as _;

fn io_err() -> Result<u8, std::io::Error> {
    Err(std::io::Error::other("disk on fire"))
}

/// Own domain error that does not implement `std::error::Error`.
struct Teapot;

impl From<Teapot> for BrestErr {
    fn from(_: Teapot) -> Self {
        BrestErr::fail("short and stout").with_status(StatusCode::IM_A_TEAPOT)
    }
}

fn teapot() -> Result<u8, Teapot> {
    Err(Teapot)
}

async fn plain() -> Result<Brest<u8>, BrestErr> {
    Ok(Brest::success(io_err()?))
}

async fn not_found() -> Result<Brest<u8>, BrestErr> {
    Ok(Brest::success(io_err().status(StatusCode::NOT_FOUND)?))
}

async fn failing() -> Result<Brest<u8>, BrestErr> {
    Ok(Brest::success(io_err().fail()?))
}

async fn coded() -> Result<Brest<u8>, BrestErr> {
    Ok(Brest::success(
        io_err().fail().code(7).status(StatusCode::CONFLICT)?,
    ))
}

async fn custom_message() -> Result<Brest<u8>, BrestErr> {
    Ok(Brest::success(io_err().message("nope")?))
}

async fn own_type_fail() -> Result<Brest<u8>, BrestErr> {
    Ok(Brest::success(teapot().fail()?))
}

async fn own_type() -> Result<Brest<u8>, BrestErr> {
    Ok(Brest::success(teapot()?))
}

async fn ok() -> Result<Brest<u8>, BrestErr> {
    Ok(Brest::success(1))
}

#[cfg(feature = "try")]
async fn direct() -> Brest<u8> {
    Brest::success(io_err()?)
}

#[cfg(feature = "try")]
async fn direct_nested() -> Brest<u8> {
    let inner: Brest<u8> = Brest::fail("inner").with_status(StatusCode::FORBIDDEN);
    Brest::success(inner? + 1)
}

#[cfg(feature = "try")]
async fn direct_overridden() -> Brest<u8> {
    Brest::success(io_err().status(StatusCode::NOT_FOUND)?)
}

async fn call(path: &str) -> (StatusCode, Value) {
    let app = Router::new()
        .route("/plain", get(plain))
        .route("/not_found", get(not_found))
        .route("/failing", get(failing))
        .route("/coded", get(coded))
        .route("/message", get(custom_message))
        .route("/own", get(own_type))
        .route("/own_fail", get(own_type_fail))
        .route("/ok", get(ok));
    #[cfg(feature = "try")]
    let app = app
        .route("/direct", get(direct))
        .route("/direct_nested", get(direct_nested))
        .route("/direct_overridden", get(direct_overridden));
    let res = app
        .oneshot(Request::get(path).body(Body::empty()).unwrap())
        .await
        .unwrap();
    let status = res.status();
    let bytes = res.into_body().collect().await.unwrap().to_bytes();
    (status, serde_json::from_slice(&bytes).unwrap())
}

#[tokio::test]
async fn question_mark_defaults_to_500_error() {
    let (status, body) = call("/plain").await;
    assert_eq!(status, StatusCode::INTERNAL_SERVER_ERROR);
    assert_eq!(body, json!({"type": "error", "message": "disk on fire"}));
}

#[tokio::test]
async fn status_override() {
    let (status, body) = call("/not_found").await;
    assert_eq!(status, StatusCode::NOT_FOUND);
    assert_eq!(body["type"], "error");
}

#[tokio::test]
async fn fail_is_400() {
    let (status, body) = call("/failing").await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert_eq!(body, json!({"type": "fail", "message": "disk on fire"}));
}

#[tokio::test]
async fn chaining_fail_code_status() {
    let (status, body) = call("/coded").await;
    assert_eq!(status, StatusCode::CONFLICT);
    assert_eq!(
        body,
        json!({"type": "fail", "message": "disk on fire", "code": 7})
    );
}

#[tokio::test]
async fn message_override() {
    let (status, body) = call("/message").await;
    assert_eq!(status, StatusCode::INTERNAL_SERVER_ERROR);
    assert_eq!(body, json!({"type": "error", "message": "nope"}));
}

#[tokio::test]
async fn own_type_picks_status() {
    let (status, body) = call("/own").await;
    assert_eq!(status, StatusCode::IM_A_TEAPOT);
    assert_eq!(body, json!({"type": "fail", "message": "short and stout"}));
}

#[tokio::test]
async fn fail_keeps_status_of_existing_fail() {
    let (status, body) = call("/own_fail").await;
    assert_eq!(status, StatusCode::IM_A_TEAPOT);
    assert_eq!(body["type"], "fail");
}

#[tokio::test]
async fn success_untouched() {
    let (status, body) = call("/ok").await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body, json!({"type": "success", "data": 1}));
}

#[cfg(feature = "try")]
#[tokio::test]
async fn direct_brest_return() {
    let (status, body) = call("/direct").await;
    assert_eq!(status, StatusCode::INTERNAL_SERVER_ERROR);
    assert_eq!(body, json!({"type": "error", "message": "disk on fire"}));

    let (status, body) = call("/direct_nested").await;
    assert_eq!(status, StatusCode::FORBIDDEN);
    assert_eq!(body, json!({"type": "fail", "message": "inner"}));

    let (status, _) = call("/direct_overridden").await;
    assert_eq!(status, StatusCode::NOT_FOUND);
}
