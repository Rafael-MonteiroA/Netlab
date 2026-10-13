use futures_util::{SinkExt, StreamExt};
use serde_json::{json, Value};
use std::time::Duration;
use tokio::{
    net::{TcpListener, TcpStream},
    task::JoinHandle,
};
use tokio_tungstenite::{connect_async, tungstenite::Message, MaybeTlsStream, WebSocketStream};

type Client = WebSocketStream<MaybeTlsStream<TcpStream>>;

#[tokio::test]
async fn rejects_cross_origin_browser_connections() {
    use tokio_tungstenite::tungstenite::client::IntoClientRequest;
    let server = server().await;
    let mut request = server.url.clone().into_client_request().unwrap();
    request
        .headers_mut()
        .insert("origin", "https://unrelated.example".parse().unwrap());
    let result = connect_async(request).await;
    match result {
        Err(tokio_tungstenite::tungstenite::Error::Http(response)) => {
            assert_eq!(response.status().as_u16(), 403)
        }
        _ => panic!("cross-origin connection must be rejected"),
    }
}
struct Server {
    url: String,
    task: JoinHandle<()>,
}
impl Drop for Server {
    fn drop(&mut self) {
        self.task.abort();
    }
}
async fn server() -> Server {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let url = format!("ws://{}/ws", listener.local_addr().unwrap());
    let task = tokio::spawn(async {
        axum::serve(listener, netlab_server::app()).await.unwrap();
    });
    Server { url, task }
}
async fn command(client: &mut Client, data: Value) {
    client
        .send(Message::Text(data.to_string().into()))
        .await
        .unwrap();
}
async fn event(client: &mut Client, kind: &str) -> Value {
    tokio::time::timeout(Duration::from_secs(3), async {
        loop {
            let msg = client
                .next()
                .await
                .expect("socket closed")
                .expect("websocket error");
            if let Message::Text(text) = msg {
                let value: Value = serde_json::from_str(&text).unwrap();
                if value["type"] == kind {
                    return value;
                }
            }
        }
    })
    .await
    .expect("event timed out")
}
async fn connect(server: &Server, data: Value) -> (Client, Value) {
    let (mut client, _) = connect_async(&server.url).await.unwrap();
    command(&mut client, data).await;
    let welcome = event(&mut client, "welcome").await;
    (client, welcome)
}
async fn assert_no_chat(client: &mut Client) {
    assert!(tokio::time::timeout(Duration::from_millis(150), async {
        loop {
            if let Some(Ok(Message::Text(text))) = client.next().await {
                let value: Value = serde_json::from_str(&text).unwrap();
                assert_ne!(value["type"], "chat", "private message leaked");
            } else {
                panic!("socket closed");
            }
        }
    })
    .await
    .is_err());
}

#[tokio::test]
async fn three_clients_exchange_broadcast_private_messages_and_peer_ping() {
    let server = server().await;
    let (mut a, wa) = connect(
        &server,
        json!({"type":"create","name":"Notebook","room_name":"Casa"}),
    )
    .await;
    let (mut b, wb) = connect(
        &server,
        json!({"type":"join","name":"Celular","code":wa["code"]}),
    )
    .await;
    let (mut c, _) = connect(
        &server,
        json!({"type":"join","name":"PC","code":wa["code"]}),
    )
    .await;
    command(
        &mut a,
        json!({"type":"chat","text":"Olá, rede!","to":null,"from":"spoof"}),
    )
    .await;
    for client in [&mut a, &mut b, &mut c] {
        let msg = event(client, "chat").await;
        assert_eq!(msg["text"], "Olá, rede!");
        assert_eq!(msg["from"], wa["id"]);
    }
    command(
        &mut a,
        json!({"type":"chat","text":"Só para o celular","to":wb["id"]}),
    )
    .await;
    assert_eq!(event(&mut b, "chat").await["text"], "Só para o celular");
    assert_eq!(event(&mut a, "chat").await["forwarded"], 1);
    assert_no_chat(&mut c).await;
    command(
        &mut a,
        json!({"type":"ping","to":wb["id"],"nonce":"round-trip-1"}),
    )
    .await;
    let ping = event(&mut b, "ping").await;
    assert_eq!(ping["from"], wa["id"]);
    command(
        &mut b,
        json!({"type":"pong","to":ping["from"],"nonce":ping["nonce"]}),
    )
    .await;
    let pong = event(&mut a, "pong").await;
    assert_eq!(pong["from"], wb["id"]);
    assert_eq!(pong["nonce"], "round-trip-1");
}

#[tokio::test]
async fn rooms_are_isolated_and_invalid_invites_are_rejected() {
    let server = server().await;
    let (mut a, _) = connect(
        &server,
        json!({"type":"create","name":"A","room_name":"Sala A"}),
    )
    .await;
    let (mut b, wb) = connect(
        &server,
        json!({"type":"create","name":"B","room_name":"Sala B"}),
    )
    .await;
    command(&mut a, json!({"type":"chat","text":"Privado","to":null})).await;
    assert_eq!(event(&mut a, "chat").await["forwarded"], 0);
    assert_no_chat(&mut b).await;
    command(
        &mut a,
        json!({"type":"chat","text":"Tentativa entre salas","to":wb["id"]}),
    )
    .await;
    assert_eq!(
        event(&mut a, "error").await["message"],
        "O destinatário não está conectado."
    );
    let (mut invalid, _) = connect_async(&server.url).await.unwrap();
    command(
        &mut invalid,
        json!({"type":"join","name":"Intruso","code":"errado"}),
    )
    .await;
    assert!(event(&mut invalid, "error").await["message"]
        .as_str()
        .unwrap()
        .contains("Convite inválido"));
}

#[tokio::test]
async fn disconnect_updates_presence_and_empty_room_can_reconnect() {
    let server = server().await;
    let (mut a, wa) = connect(
        &server,
        json!({"type":"create","name":"A","room_name":"Reconexão"}),
    )
    .await;
    let (mut b, _) = connect(&server, json!({"type":"join","name":"B","code":wa["code"]})).await;
    // Consume the two-client presence before checking the leave event.
    while event(&mut a, "peers").await["peers"]
        .as_array()
        .unwrap()
        .len()
        != 2
    {}
    b.close(None).await.unwrap();
    tokio::time::timeout(Duration::from_secs(3), async {
        while event(&mut a, "peers").await["peers"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|p| p["online"] == true)
            .count()
            != 1
        {}
    })
    .await
    .expect("leave presence");
    a.close(None).await.unwrap();
    let (_, rejoined) = connect(
        &server,
        json!({"type":"join","name":"A novamente","code":wa["code"]}),
    )
    .await;
    assert_eq!(rejoined["room_name"], "Reconexão");
    assert_ne!(rejoined["id"], wa["id"]);
}

#[tokio::test]
async fn reconnect_preserves_identity_and_replays_only_authorized_messages() {
    let server = server().await;
    let (mut a, wa) = connect(
        &server,
        json!({"type":"create","name":"A","room_name":"Reliable"}),
    )
    .await;
    let (mut b, wb) = connect(&server, json!({"type":"join","name":"B","code":wa["code"]})).await;
    b.close(None).await.unwrap();
    command(
        &mut a,
        json!({"type":"chat","text":"Durante queda","to":wb["id"],"client_seq":1}),
    )
    .await;
    let sent = event(&mut a, "chat").await;
    let (mut restored, welcome) = connect(
        &server,
        json!({"type":"resume","code":wa["code"],"token":wb["resume_token"]}),
    )
    .await;
    assert_eq!(welcome["id"], wb["id"]);
    let replay = event(&mut restored, "chat").await;
    assert_eq!(replay["id"], sent["id"]);
    command(
        &mut restored,
        json!({"type":"receipt","id":sent["id"],"read":true}),
    )
    .await;
    loop {
        let receipt = event(&mut a, "receipt").await;
        if receipt["read"] == 1 {
            assert_eq!(receipt["received"], 1);
            break;
        }
    }
    let (mut c, _) = connect(&server, json!({"type":"join","name":"C","code":wa["code"]})).await;
    assert_no_chat(&mut c).await;
}

#[tokio::test]
async fn duplicate_sequence_does_not_redeliver_and_resume_requires_private_token() {
    let server = server().await;
    let (mut a, wa) = connect(
        &server,
        json!({"type":"create","name":"A","room_name":"Dedup"}),
    )
    .await;
    let (mut b, _) = connect(&server, json!({"type":"join","name":"B","code":wa["code"]})).await;
    let message = json!({"type":"chat","text":"Uma vez","client_seq":1});
    command(&mut a, message.clone()).await;
    let first = event(&mut a, "chat").await;
    event(&mut b, "chat").await;
    command(
        &mut b,
        json!({"type":"receipt","id":first["id"],"read":false}),
    )
    .await;
    command(&mut a, message).await;
    let retry = event(&mut a, "chat").await;
    assert_eq!(first["id"], retry["id"]);
    assert_no_chat(&mut b).await;
    let (mut intruder, _) = connect_async(&server.url).await.unwrap();
    command(
        &mut intruder,
        json!({"type":"resume","code":wa["code"],"token":wa["id"]}),
    )
    .await;
    assert!(event(&mut intruder, "error").await["message"]
        .as_str()
        .unwrap()
        .contains("Sessão expirada"));
}

#[tokio::test]
async fn rejects_invalid_and_oversized_text_without_breaking_session() {
    let server = server().await;
    let (mut a, _) = connect(
        &server,
        json!({"type":"create","name":"A","room_name":"Validação"}),
    )
    .await;
    for text in [" ".to_string(), "x".repeat(4001)] {
        command(
            &mut a,
            json!({"type":"chat","text":text,"to":null,"client_seq":7}),
        )
        .await;
        assert_eq!(event(&mut a, "error").await["client_seq"], 7);
    }
    a.send(Message::Text("not json".into())).await.unwrap();
    event(&mut a, "error").await;
    command(&mut a, json!({"type":"heartbeat"})).await;
    event(&mut a, "heartbeat").await;
}

#[tokio::test]
async fn evicted_sequence_is_rejected_instead_of_delivered_twice() {
    let server = server().await;
    let (mut a, _) = connect(
        &server,
        json!({"type":"create","name":"A","room_name":"Window"}),
    )
    .await;
    for seq in 1..=257 {
        command(
            &mut a,
            json!({"type":"chat","text":"History","client_seq":seq}),
        )
        .await;
        assert_eq!(event(&mut a, "chat").await["client_seq"], seq);
    }
    command(
        &mut a,
        json!({"type":"chat","text":"Old retry","client_seq":1}),
    )
    .await;
    assert_eq!(event(&mut a, "expired").await["client_seq"], 1);
    assert_no_chat(&mut a).await;
}

#[tokio::test]
async fn unrelated_member_cannot_acknowledge_private_message() {
    let server = server().await;
    let (mut a, wa) = connect(
        &server,
        json!({"type":"create","name":"A","room_name":"Receipts"}),
    )
    .await;
    let (mut b, wb) = connect(&server, json!({"type":"join","name":"B","code":wa["code"]})).await;
    let (mut c, _) = connect(&server, json!({"type":"join","name":"C","code":wa["code"]})).await;
    command(
        &mut a,
        json!({"type":"chat","text":"Private","to":wb["id"],"client_seq":1}),
    )
    .await;
    let sent = event(&mut a, "chat").await;
    assert_eq!(event(&mut a, "receipt").await["received"], 0);
    event(&mut b, "chat").await;
    command(
        &mut c,
        json!({"type":"receipt","id":sent["id"],"read":true}),
    )
    .await;
    command(&mut c, json!({"type":"heartbeat"})).await;
    event(&mut c, "heartbeat").await; // Ensure the forged receipt was processed.
    command(
        &mut a,
        json!({"type":"chat","text":"Private","to":wb["id"],"client_seq":1}),
    )
    .await;
    let status = event(&mut a, "receipt").await;
    assert_eq!(status["received"], 0);
    assert_eq!(status["read"], 0);
}

#[tokio::test]
async fn limits_room_capacity() {
    let server = server().await;
    let (a, wa) = connect(
        &server,
        json!({"type":"create","name":"A","room_name":"Limite"}),
    )
    .await;
    let mut clients = vec![a];
    for index in 1..16 {
        clients.push(
            connect(
                &server,
                json!({"type":"join","name":format!("PC{index}"),"code":wa["code"]}),
            )
            .await
            .0,
        );
    }
    let (mut overflow, _) = connect_async(&server.url).await.unwrap();
    command(
        &mut overflow,
        json!({"type":"join","name":"PC17","code":wa["code"]}),
    )
    .await;
    assert!(event(&mut overflow, "error").await["message"]
        .as_str()
        .unwrap()
        .contains("16 aparelhos"));
}
