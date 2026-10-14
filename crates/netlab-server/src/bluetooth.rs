//! Local-only bridge from the web UI to an explicitly selected paired Bluetooth device.
use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        ConnectInfo,
    },
    http::{HeaderMap, StatusCode},
    response::{IntoResponse, Response},
    Json,
};
use serde_json::{json, Value};
use std::{net::SocketAddr, time::Duration};
use tokio::sync::{mpsc, Semaphore};
static CONNECTION: Semaphore = Semaphore::const_new(1);
pub const SERVICE_UUID: &str = "08d4a386-30a7-4bd7-aeca-958c6432f208";
fn invite_uri(address: &str) -> String {
    format!("netlab://bluetooth/v1?pc={address}")
}
pub async fn qr_invite(
    ConnectInfo(remote): ConnectInfo<SocketAddr>,
    headers: HeaderMap,
) -> Response {
    if !allowed(remote, &headers) {
        return StatusCode::FORBIDDEN.into_response();
    }
    match tokio::task::spawn_blocking(platform::radio).await {
        Ok(Ok((address, name))) => {
            let uri = invite_uri(&address);
            match qrcode::QrCode::new(uri.as_bytes()) {
                Ok(code) => Json(json!({"name":name,"address":address,"uri":uri,
                    "svg":code.render::<qrcode::render::svg::Color>().min_dimensions(280,280).quiet_zone(true).build()})).into_response(),
                Err(_) => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
            }
        }
        _ => (StatusCode::SERVICE_UNAVAILABLE, Json(json!({"error":"Ative um adaptador Bluetooth no PC e pareie o celular antes de gerar o QR Code."}))).into_response(),
    }
}
pub(crate) fn allowed(remote: SocketAddr, headers: &HeaderMap) -> bool {
    let host = headers
        .get("host")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");
    remote.ip().is_loopback()
        && ["localhost:", "127.0.0.1:", "[::1]:"]
            .iter()
            .any(|v| host.starts_with(v))
        && crate::reliable::same_origin(headers)
}
pub async fn devices(ConnectInfo(remote): ConnectInfo<SocketAddr>, headers: HeaderMap) -> Response {
    if !allowed(remote, &headers) {
        return (
            StatusCode::FORBIDDEN,
            "Abra esta função por http://localhost no PC.",
        )
            .into_response();
    }
    match tokio::task::spawn_blocking(platform::devices).await{
  Ok(Ok(devices))=>Json(json!({"devices":devices,"uuid":SERVICE_UUID})).into_response(),
  _=>Json(json!({"devices":[],"error":"Bluetooth indisponível. Ative um adaptador e pareie o Android nas configurações do Windows."})).into_response()
 }
}
pub async fn upgrade(
    ws: WebSocketUpgrade,
    ConnectInfo(remote): ConnectInfo<SocketAddr>,
    headers: HeaderMap,
) -> Response {
    if !allowed(remote, &headers) {
        return StatusCode::FORBIDDEN.into_response();
    }
    let Ok(permit) = CONNECTION.try_acquire() else {
        return (
            StatusCode::CONFLICT,
            "Bluetooth já está em uso por outra aba.",
        )
            .into_response();
    };
    ws.max_message_size(16384)
        .on_upgrade(move |socket| async move {
            let _permit = permit;
            session(socket).await;
        })
}
async fn session(mut socket: WebSocket) {
    let Ok(Some(Ok(Message::Text(text)))) =
        tokio::time::timeout(Duration::from_secs(10), socket.recv()).await
    else {
        return;
    };
    let Ok(value) = serde_json::from_str::<Value>(&text) else {
        return;
    };
    let Some(address) = value
        .get("address")
        .and_then(Value::as_str)
        .map(str::to_owned)
    else {
        return;
    };
    let (tx, rx) = mpsc::channel::<String>(64);
    let (events, mut received) = mpsc::channel::<Value>(64);
    let task = tokio::task::spawn_blocking(move || {
        if let Err(message) = platform::run(&address, rx, &events) {
            let _ = events.try_send(json!({"type":"error","message":message}));
        }
    });
    loop {
        tokio::select! {
         event=received.recv()=>{let Some(event)=event else{break;};if !matches!(tokio::time::timeout(Duration::from_secs(5),socket.send(Message::Text(event.to_string().into()))).await,Ok(Ok(()))){break;}}
         frame=socket.recv()=>{match frame{Some(Ok(Message::Text(text)))=>{if valid_frame(&text)&&tx.try_send(text.to_string()).is_err(){break;}},Some(Ok(Message::Ping(_)|Message::Pong(_)))=>{},_=>break}}
        }
    }
    drop(tx);
    let _ = tokio::time::timeout(Duration::from_secs(2), task).await;
}
pub(crate) fn valid_frame(text: &str) -> bool {
    if text.len() > 16384 {
        return false;
    }
    let Ok(v) = serde_json::from_str::<Value>(text) else {
        return false;
    };
    let valid_id = v["id"].as_str().is_some_and(|s| {
        !s.is_empty() && s.len() <= 64 && s.chars().all(|c| c.is_ascii_alphanumeric() || c == '-')
    });
    match v["type"].as_str() {
        Some("chat") => {
            valid_id
                && v["text"]
                    .as_str()
                    .is_some_and(|s| !s.trim().is_empty() && s.chars().count() <= 4000)
                && v["name"].as_str().is_some_and(|s| s.chars().count() <= 40)
        }
        Some("ack") => valid_id && matches!(v["status"].as_str(), Some("received" | "read")),
        Some("hello") => v["name"].as_str().is_some_and(|s| s.chars().count() <= 40),
        Some("ping" | "pong") => valid_id,
        _ => false,
    }
}
#[cfg(not(windows))]
mod platform {
    use super::*;
    pub fn radio() -> Result<(String, String), String> {
        Err("Bluetooth exige Windows.".into())
    }
    pub fn devices() -> Result<Vec<Value>, String> {
        Err("A conexão Bluetooth deste servidor exige Windows.".into())
    }
    pub fn run(_: &str, _: mpsc::Receiver<String>, _: &mpsc::Sender<Value>) -> Result<(), String> {
        Err("Bluetooth disponível no Windows.".into())
    }
}
#[cfg(windows)]
mod platform {
    use super::*;
    use std::{mem::size_of, ptr::null_mut, time::Instant};
    use windows_sys::{
        core::GUID,
        Win32::{Devices::Bluetooth::*, Foundation::CloseHandle, Networking::WinSock::*},
    };
    pub fn radio() -> Result<(String, String), String> {
        unsafe {
            let params = BLUETOOTH_FIND_RADIO_PARAMS {
                dwSize: size_of::<BLUETOOTH_FIND_RADIO_PARAMS>() as u32,
            };
            let mut handle = null_mut();
            let find = BluetoothFindFirstRadio(&params, &mut handle);
            if find.is_null() {
                return Err("Nenhum rádio Bluetooth ativo.".into());
            }
            let mut info = BLUETOOTH_RADIO_INFO {
                dwSize: size_of::<BLUETOOTH_RADIO_INFO>() as u32,
                ..Default::default()
            };
            let result = BluetoothGetRadioInfo(handle, &mut info);
            BluetoothFindRadioClose(find);
            CloseHandle(handle);
            if result != 0 {
                return Err("Não foi possível identificar o adaptador Bluetooth.".into());
            }
            let value = info.address.Anonymous.ullLong;
            let address = (0..6)
                .rev()
                .map(|i| format!("{:02X}", (value >> (i * 8)) & 255))
                .collect::<Vec<_>>()
                .join(":");
            let name = String::from_utf16_lossy(
                &info.szName[..info.szName.iter().position(|c| *c == 0).unwrap_or(248)],
            );
            Ok((address, name))
        }
    }
    pub fn devices() -> Result<Vec<Value>, String> {
        // All FFI buffers use generated Windows layouts; handles are closed in this scope.
        unsafe {
            let params = BLUETOOTH_FIND_RADIO_PARAMS {
                dwSize: size_of::<BLUETOOTH_FIND_RADIO_PARAMS>() as u32,
            };
            let mut radio = null_mut();
            let find = BluetoothFindFirstRadio(&params, &mut radio);
            if find.is_null() {
                return Err("Nenhum rádio Bluetooth ativo.".into());
            }
            BluetoothFindRadioClose(find);
            CloseHandle(radio);
            let search = BLUETOOTH_DEVICE_SEARCH_PARAMS {
                dwSize: size_of::<BLUETOOTH_DEVICE_SEARCH_PARAMS>() as u32,
                fReturnAuthenticated: 1,
                fReturnRemembered: 1,
                ..Default::default()
            };
            let mut info = BLUETOOTH_DEVICE_INFO {
                dwSize: size_of::<BLUETOOTH_DEVICE_INFO>() as u32,
                ..Default::default()
            };
            let find = BluetoothFindFirstDevice(&search, &mut info);
            let mut out = Vec::new();
            if !find.is_null() {
                loop {
                    let addr = info.Address.Anonymous.ullLong;
                    let address = (0..6)
                        .rev()
                        .map(|i| format!("{:02X}", (addr >> (i * 8)) & 255))
                        .collect::<Vec<_>>()
                        .join(":");
                    let name = String::from_utf16_lossy(
                        &info.szName[..info.szName.iter().position(|c| *c == 0).unwrap_or(248)],
                    );
                    out.push(json!({"name":name,"address":address}));
                    if BluetoothFindNextDevice(find, &mut info) == 0 {
                        break;
                    }
                }
                BluetoothFindDeviceClose(find);
            }
            Ok(out)
        }
    }
    struct Winsock;
    impl Drop for Winsock {
        fn drop(&mut self) {
            unsafe {
                WSACleanup();
            }
        }
    }
    struct Socket(SOCKET);
    impl Drop for Socket {
        fn drop(&mut self) {
            unsafe {
                closesocket(self.0);
            }
        }
    }
    fn failure() -> String {
        format!(
            "Falha Bluetooth (Windows {}). Confira pareamento e 'Receber conexão' no Android.",
            unsafe { WSAGetLastError() }
        )
    }
    pub fn run(
        address: &str,
        mut commands: mpsc::Receiver<String>,
        events: &mpsc::Sender<Value>,
    ) -> Result<(), String> {
        if !devices()?.iter().any(|d| d["address"] == address) {
            return Err("Selecione um dispositivo já pareado no Windows.".into());
        }
        let addr = u64::from_str_radix(&address.replace(':', ""), 16)
            .map_err(|_| "Endereço Bluetooth inválido.")?;
        unsafe {
            let mut data = WSADATA::default();
            if WSAStartup(0x202, &mut data) != 0 {
                return Err(failure());
            }
            let _winsock = Winsock;
            let raw = socket(AF_BTH as i32, SOCK_STREAM, BTHPROTO_RFCOMM as i32);
            if raw == INVALID_SOCKET {
                return Err(failure());
            }
            let socket = Socket(raw);
            let enabled: i32 = 1;
            for option in [SO_BTH_AUTHENTICATE, SO_BTH_ENCRYPT] {
                if setsockopt(
                    raw,
                    SOL_RFCOMM as i32,
                    option as i32,
                    (&enabled as *const i32).cast(),
                    4,
                ) != 0
                {
                    return Err(failure());
                }
            }
            let mut nonblocking = 1;
            if ioctlsocket(raw, FIONBIO, &mut nonblocking) != 0 {
                return Err(failure());
            }
            let target = SOCKADDR_BTH {
                addressFamily: AF_BTH,
                btAddr: addr,
                serviceClassId: GUID::from_u128(0x08d4a386_30a7_4bd7_aeca_958c6432f208),
                port: 0,
            };
            if connect(
                raw,
                (&target as *const SOCKADDR_BTH).cast(),
                size_of::<SOCKADDR_BTH>() as i32,
            ) != 0
            {
                if WSAGetLastError() != WSAEWOULDBLOCK {
                    return Err(failure());
                }
                let start = Instant::now();
                loop {
                    if commands.is_closed() {
                        return Ok(());
                    }
                    if start.elapsed() > Duration::from_secs(20) {
                        return Err("Tempo esgotado. Abra 'Receber conexão' no Android e aproxime os aparelhos.".into());
                    }
                    let mut writable = FD_SET {
                        fd_count: 1,
                        ..Default::default()
                    };
                    writable.fd_array[0] = raw;
                    let mut errors = writable;
                    let timeout = TIMEVAL {
                        tv_sec: 0,
                        tv_usec: 200000,
                    };
                    let n = select(0, null_mut(), &mut writable, &mut errors, &timeout);
                    if n < 0 {
                        return Err(failure());
                    }
                    if n > 0 {
                        let mut error = 0i32;
                        let mut len = 4;
                        if getsockopt(
                            raw,
                            SOL_SOCKET,
                            SO_ERROR,
                            (&mut error as *mut i32).cast(),
                            &mut len,
                        ) != 0
                            || error != 0
                        {
                            return Err(format!("Não foi possível conectar (código {error}). Confira o app Android."));
                        }
                        break;
                    }
                }
            }
            let _ = events.try_send(json!({"type":"connected","address":address}));
            let mut incoming = Vec::new();
            let mut outgoing = Vec::<u8>::new();
            let mut buffer = [0u8; 4096];
            let mut last_seen = Instant::now();
            let mut last_write = Instant::now();
            loop {
                if commands.is_closed() || events.is_closed() {
                    break;
                }
                if outgoing.is_empty() {
                    if let Ok(command) = commands.try_recv() {
                        outgoing = command.into_bytes();
                        outgoing.push(b'\n');
                        last_write = Instant::now();
                    }
                }
                if !outgoing.is_empty() {
                    let n = send(socket.0, outgoing.as_ptr(), outgoing.len() as i32, 0);
                    if n > 0 {
                        outgoing.drain(..n as usize);
                    } else if n < 0 && WSAGetLastError() != WSAEWOULDBLOCK {
                        return Err(failure());
                    }
                    if last_write.elapsed() > Duration::from_secs(10) {
                        return Err("Bluetooth sem resposta ao enviar.".into());
                    }
                }
                let n = recv(socket.0, buffer.as_mut_ptr(), buffer.len() as i32, 0);
                if n == 0 {
                    return Err("O outro aparelho encerrou a conexão.".into());
                }
                if n > 0 {
                    last_seen = Instant::now();
                    incoming.extend_from_slice(&buffer[..n as usize]);
                    while let Some(end) = incoming.iter().position(|b| *b == b'\n') {
                        let bytes = incoming.drain(..=end).collect::<Vec<_>>();
                        let text =
                            std::str::from_utf8(&bytes).map_err(|_| "Dados Bluetooth inválidos")?;
                        if !valid_frame(text) {
                            return Err("Quadro Bluetooth inválido.".into());
                        }
                        let v = serde_json::from_str(text).map_err(|_| "JSON inválido")?;
                        if events.try_send(v).is_err() {
                            return Err("Interface ocupada; reconecte.".into());
                        }
                    }
                    if incoming.len() > 16384 {
                        return Err("Mensagem Bluetooth muito grande.".into());
                    }
                } else if WSAGetLastError() != WSAEWOULDBLOCK {
                    return Err(failure());
                }
                if last_seen.elapsed() > Duration::from_secs(60) {
                    return Err("Conexão Bluetooth sem atividade. Reconectando.".into());
                }
                std::thread::sleep(Duration::from_millis(20));
            }
            Ok(())
        }
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn qr_is_an_offline_invite_and_local_only() {
        assert_eq!(
            invite_uri("AA:BB:CC:DD:EE:FF"),
            "netlab://bluetooth/v1?pc=AA:BB:CC:DD:EE:FF"
        );
        let mut headers = HeaderMap::new();
        headers.insert("host", "localhost:8787".parse().unwrap());
        headers.insert("origin", "http://localhost:8787".parse().unwrap());
        assert!(allowed("127.0.0.1:12345".parse().unwrap(), &headers));
        assert!(!allowed("192.168.1.5:12345".parse().unwrap(), &headers));
        headers.insert("origin", "http://example.com".parse().unwrap());
        assert!(!allowed("127.0.0.1:12345".parse().unwrap(), &headers));
    }
    #[test]
    fn protocol_rejects_unbounded_or_unknown_frames() {
        assert!(valid_frame(
            r#"{"type":"chat","id":"abc-1","name":"Android","text":"Olá"}"#
        ));
        assert!(!valid_frame(
            r#"{"type":"chat","id":"abc","name":"x","text":""}"#
        ));
        assert!(!valid_frame(r#"{"type":"run","command":"anything"}"#));
        assert!(valid_frame(r#"{"type":"ack","id":"abc","status":"read"}"#));
    }
}
