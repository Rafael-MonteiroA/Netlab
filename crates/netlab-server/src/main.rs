use std::{env, net::SocketAddr};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut bind = env::var("NETLAB_BIND").unwrap_or_else(|_| "0.0.0.0:8787".into());
    let mut args = env::args().skip(1);
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--bind" => bind = args.next().ok_or("--bind exige IP:porta")?,
            "--help" | "-h" => {
                println!("NetLab 0.4\n\nnetlab-server [--bind 0.0.0.0:8787]\n\nAbra http://localhost:8787. Em outro aparelho, use o IP deste computador.\nPara internet, use um proxy HTTPS conforme docs/real-network.md.");
                return Ok(());
            }
            _ => return Err(format!("Argumento desconhecido: {arg}").into()),
        }
    }
    let address: SocketAddr = bind.parse()?;
    let listener = tokio::net::TcpListener::bind(address).await?;
    println!("\n  NETLAB 0.4 — Laboratório e comunicação real\n  Escutando em {}\n  Neste computador: http://localhost:{}\n  Outros aparelhos: http://<IP-DESTE-COMPUTADOR>:{}\n  Ctrl+C para encerrar.\n", listener.local_addr()?, address.port(), address.port());
    axum::serve(
        listener,
        netlab_server::app().into_make_service_with_connect_info::<SocketAddr>(),
    )
    .with_graceful_shutdown(async {
        let _ = tokio::signal::ctrl_c().await;
    })
    .await?;
    Ok(())
}
