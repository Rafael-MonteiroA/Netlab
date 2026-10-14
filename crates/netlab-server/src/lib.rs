mod bluetooth;
mod network_info;
mod reliable;
mod simulation;
mod vault;

use axum::{
    extract::{DefaultBodyLimit, Request},
    http::{header, HeaderValue},
    middleware::{self, Next},
    response::{Html, IntoResponse, Response},
    routing::{get, post},
    Router,
};

pub fn app() -> Router {
    Router::new()
        .route(
            "/",
            get(|| async { Html(include_str!("../web/index.html")) }),
        )
        .route(
            "/app.js",
            get(|| async {
                (
                    [(header::CONTENT_TYPE, "text/javascript; charset=utf-8")],
                    include_str!("../web/app.js"),
                )
            }),
        )
        .route(
            "/style.css",
            get(|| async {
                (
                    [(header::CONTENT_TYPE, "text/css; charset=utf-8")],
                    include_str!("../web/style.css"),
                )
            }),
        )
        .route(
            "/api/health",
            get(|| async {
                axum::Json(serde_json::json!({"status":"ok", "version":env!("CARGO_PKG_VERSION")}))
            }),
        )
        .route("/api/simulate", post(simulation::simulate))
        .route("/api/network", get(network_info::info))
        .route(
            "/vault-client.js",
            get(|| async {
                (
                    [(header::CONTENT_TYPE, "text/javascript; charset=utf-8")],
                    include_str!("../web/vault-client.js"),
                )
            }),
        )
        .route("/api/vault", get(vault::read).post(vault::write))
        .route("/api/bluetooth/devices", get(bluetooth::devices))
        .route("/api/bluetooth/qr", get(bluetooth::qr_invite))
        .route("/ws/bluetooth", get(bluetooth::upgrade))
        .route(
            "/bluetooth-client.js",
            get(|| async {
                (
                    [(header::CONTENT_TYPE, "text/javascript; charset=utf-8")],
                    include_str!("../web/bluetooth-client.js"),
                )
            }),
        )
        .route(
            "/network-client.js",
            get(|| async {
                (
                    [(header::CONTENT_TYPE, "text/javascript; charset=utf-8")],
                    include_str!("../web/network-client.js"),
                )
            }),
        )
        .route("/api/qr", post(network_info::qr))
        .route("/ws", get(reliable::upgrade))
        .layer(DefaultBodyLimit::max(32 * 1024))
        .layer(middleware::from_fn(headers))
        .with_state(reliable::Hub::default())
}

async fn headers(request: Request, next: Next) -> Response {
    let mut response = next.run(request).await.into_response();
    for (key, value) in [
        ("x-content-type-options", "nosniff"),
        ("referrer-policy", "no-referrer"),
        ("cache-control", "no-store"),
        ("content-security-policy", "default-src 'self'; connect-src 'self'; style-src 'self'; script-src 'self'; img-src 'self' data: blob:; frame-ancestors 'none'; base-uri 'none'; form-action 'self'"),
    ] {
        response.headers_mut().insert(key, HeaderValue::from_static(value));
    }
    response
}
