use axum::{
    extract::ConnectInfo,
    http::{header, HeaderMap, StatusCode},
    response::IntoResponse,
    Json,
};
use serde::Deserialize;
use serde_json::json;
use std::net::SocketAddr;

pub async fn info(
    ConnectInfo(remote): ConnectInfo<SocketAddr>,
    headers: HeaderMap,
) -> impl IntoResponse {
    let host = headers
        .get("host")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("localhost:8787");
    let port = host
        .rsplit_once(':')
        .and_then(|(_, v)| v.parse::<u16>().ok())
        .unwrap_or(8787);
    let addresses = if remote.ip().is_loopback() {
        if_addrs::get_if_addrs()
            .unwrap_or_default()
            .into_iter()
            .filter(|i| !i.is_loopback() && i.ip().is_ipv4())
            .map(|i| {
                let ip = i.ip();
                let name = i.name;
                let virtual_adapter = ["virtual", "vethernet", "vpn", "wsl", "vmware"]
                    .iter()
                    .any(|s| name.to_lowercase().contains(s));
                json!({"name":name,"url":format!("http://{ip}:{port}"),"virtual":virtual_adapter})
            })
            .collect::<Vec<_>>()
    } else {
        Vec::new()
    };
    Json(
        json!({"status":"ok","version":env!("CARGO_PKG_VERSION"),"addresses":addresses,"local":remote.ip().is_loopback(),"internet_required":false,"note":"HTTP respondeu. Isso não comprova acesso de outro aparelho nem liberação do firewall."}),
    )
}
#[derive(Deserialize)]
pub struct Qr {
    url: String,
}
pub async fn qr(Json(input): Json<Qr>) -> Result<impl IntoResponse, StatusCode> {
    if input.url.len() > 512
        || !(input.url.starts_with("http://") || input.url.starts_with("https://"))
    {
        return Err(StatusCode::BAD_REQUEST);
    }
    let code = qrcode::QrCode::new(input.url.as_bytes()).map_err(|_| StatusCode::BAD_REQUEST)?;
    Ok((
        [(header::CONTENT_TYPE, "image/svg+xml")],
        code.render::<qrcode::render::svg::Color>()
            .min_dimensions(224, 224)
            .quiet_zone(true)
            .build(),
    ))
}
