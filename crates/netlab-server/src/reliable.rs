use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        State,
    },
    http::{HeaderMap, StatusCode, Uri},
    response::{IntoResponse, Response},
};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::{
    collections::{BTreeMap, HashMap, VecDeque},
    sync::Arc,
    time::{Duration, Instant},
};
use tokio::sync::{mpsc, Mutex, Semaphore};
use uuid::Uuid;
const TTL: Duration = Duration::from_secs(900);
#[derive(Clone)]
pub struct Hub {
    rooms: Arc<Mutex<HashMap<String, Room>>>,
    sockets: Arc<Semaphore>,
}
impl Default for Hub {
    fn default() -> Self {
        Self {
            rooms: Arc::default(),
            sockets: Arc::new(Semaphore::new(64)),
        }
    }
}
struct Room {
    name: String,
    peers: BTreeMap<String, Peer>,
    messages: VecDeque<Delivery>,
    touched: Instant,
}
struct Peer {
    info: PeerInfo,
    token: String,
    connection: String,
    tx: Option<mpsc::Sender<Value>>,
    last_seq: u64,
    seen: Instant,
}
#[derive(Clone, Serialize)]
struct PeerInfo {
    id: String,
    name: String,
    online: bool,
}
struct Delivery {
    event: Value,
    sender: String,
    seq: u64,
    recipients: BTreeMap<String, u8>,
}
impl Delivery {
    fn status(&self) -> Value {
        json!({"type":"receipt","id":self.event["id"],"client_seq":self.seq,"total":self.recipients.len(),"received":self.recipients.values().filter(|s|**s>=1).count(),"read":self.recipients.values().filter(|s|**s>=2).count()})
    }
}
#[derive(Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
enum Client {
    Create {
        name: String,
        room_name: String,
    },
    Join {
        name: String,
        code: String,
    },
    Resume {
        code: String,
        token: String,
    },
    Chat {
        text: String,
        to: Option<String>,
        client_seq: Option<u64>,
    },
    Receipt {
        id: String,
        read: bool,
    },
    Ping {
        to: String,
        nonce: String,
    },
    Pong {
        to: String,
        nonce: String,
    },
    Heartbeat,
}
pub(crate) fn same_origin(headers: &HeaderMap) -> bool {
    headers.get("origin").is_none_or(|origin| {
        origin
            .to_str()
            .ok()
            .and_then(|s| s.parse::<Uri>().ok())
            .is_some_and(|uri| {
                matches!(uri.scheme_str(), Some("http" | "https"))
                    && uri.authority().map(|a| a.as_str())
                        == headers.get("host").and_then(|h| h.to_str().ok())
            })
    })
}
pub async fn upgrade(ws: WebSocketUpgrade, State(hub): State<Hub>, headers: HeaderMap) -> Response {
    if !same_origin(&headers) {
        return (StatusCode::FORBIDDEN, "Origem não permitida").into_response();
    }
    let Ok(permit) = hub.sockets.clone().try_acquire_owned() else {
        return StatusCode::SERVICE_UNAVAILABLE.into_response();
    };
    ws.max_message_size(16384)
        .max_frame_size(16384)
        .on_upgrade(move |socket| async move {
            let _permit = permit;
            session(socket, hub).await;
        })
}
fn error(message: &str) -> Value {
    json!({"type":"error","message":message})
}
fn valid_name(name: &str) -> bool {
    !name.trim().is_empty() && name.chars().count() <= 40 && !name.chars().any(char::is_control)
}
fn emit(peer: &Peer, value: Value) {
    if let Some(tx) = &peer.tx {
        let _ = tx.try_send(value);
    }
}
fn presence(room: &Room) {
    let event =
        json!({"type":"peers","peers":room.peers.values().map(|p|&p.info).collect::<Vec<_>>()});
    for p in room.peers.values() {
        emit(p, event.clone());
    }
}
async fn send(socket: &mut WebSocket, value: Value) -> bool {
    matches!(
        tokio::time::timeout(
            Duration::from_secs(5),
            socket.send(Message::Text(value.to_string().into()))
        )
        .await,
        Ok(Ok(()))
    )
}
async fn session(mut socket: WebSocket, hub: Hub) {
    let Ok(Some(Ok(Message::Text(text)))) =
        tokio::time::timeout(Duration::from_secs(10), socket.recv()).await
    else {
        return;
    };
    let Ok(command) = serde_json::from_str::<Client>(&text) else {
        send(&mut socket, error("Solicitação inválida.")).await;
        return;
    };
    let (tx, mut rx) = mpsc::channel(512);
    let connection = Uuid::new_v4().simple().to_string();
    let (code, id, welcome) = match join(&hub, command, &connection, tx).await {
        Ok(v) => v,
        Err(message) => {
            send(&mut socket, error(message)).await;
            return;
        }
    };
    if send(&mut socket, welcome).await {
        let mut tick = tokio::time::interval(Duration::from_secs(15));
        tick.tick().await; // Consume the immediate tick; retries start after 15 seconds.
        let mut last_seen = Instant::now();
        let mut window = Instant::now();
        let mut count = 0;
        loop {
            tokio::select! {
             event=rx.recv()=>{let Some(event)=event else{break;};let replaced=event["type"]=="replaced";if !send(&mut socket,event).await||replaced{break;}}
             message=socket.recv()=>{let Some(Ok(message))=message else{break;};last_seen=Instant::now();if window.elapsed()>=Duration::from_secs(10){window=Instant::now();count=0;}count+=1;if count>600{send(&mut socket,error("Muitas mensagens. Aguarde e reconecte.")).await;break;}
              match message{Message::Text(text)=>{let result=match serde_json::from_str(&text){Ok(cmd)=>relay(&hub,&code,&id,&connection,cmd).await,Err(_)=>Some(error("Mensagem inválida."))};if let Some(v)=result{if !send(&mut socket,v).await{break;}}},Message::Close(_)=>break,Message::Binary(_)=>break,_=>{}}
             }
             _=tick.tick()=>{if last_seen.elapsed()>Duration::from_secs(45){break;}let rooms=hub.rooms.lock().await;if let Some(room)=rooms.get(&code){presence(room);if let Some(peer)=room.peers.get(&id){for d in &room.messages{if d.recipients.get(&id)==Some(&0){emit(peer,d.event.clone());}else if d.sender==id{emit(peer,d.status());}}}}}
            }
        }
    }
    let mut rooms = hub.rooms.lock().await;
    if let Some(room) = rooms.get_mut(&code) {
        if let Some(peer) = room.peers.get_mut(&id) {
            if peer.connection == connection {
                peer.tx = None;
                peer.info.online = false;
                peer.seen = Instant::now();
            }
        }
        room.touched = Instant::now();
        presence(room);
    }
}
async fn join(
    hub: &Hub,
    command: Client,
    connection: &str,
    tx: mpsc::Sender<Value>,
) -> Result<(String, String, Value), &'static str> {
    let mut rooms = hub.rooms.lock().await;
    rooms.retain(|_, r| r.peers.values().any(|p| p.info.online) || r.touched.elapsed() < TTL);
    let (code, name, token) = match command {
        Client::Create { name, room_name } => {
            if !valid_name(&name) || !valid_name(&room_name) {
                return Err("Use nomes de 1 a 40 caracteres.");
            }
            if rooms.len() >= 32 {
                return Err("Limite de redes atingido. Tente mais tarde.");
            }
            let code = Uuid::new_v4().simple().to_string();
            rooms.insert(
                code.clone(),
                Room {
                    name: room_name.trim().into(),
                    peers: BTreeMap::new(),
                    messages: VecDeque::new(),
                    touched: Instant::now(),
                },
            );
            (code, Some(name), None)
        }
        Client::Join { name, code } => {
            if !valid_name(&name) {
                return Err("Use um nome de 1 a 40 caracteres.");
            }
            (code.trim().to_lowercase(), Some(name), None)
        }
        Client::Resume { code, token } => (code, None, Some(token)),
        _ => return Err("Crie uma rede ou use um convite primeiro."),
    };
    let room = rooms
        .get_mut(&code)
        .ok_or("Convite inválido ou expirado. Verifique também o endereço do servidor.")?;
    room.peers
        .retain(|_, p| p.info.online || p.seen.elapsed() < TTL);
    let id = if let Some(token) = token {
        room.peers
            .iter()
            .find(|(_, p)| p.token == token)
            .map(|(id, _)| id.clone())
            .ok_or("Sessão expirada. Entre novamente pelo convite.")?
    } else {
        if room.peers.len() >= 16 {
            return Err("Esta rede já tem 16 aparelhos (incluindo reconexões reservadas).");
        }
        let id = Uuid::new_v4().simple().to_string();
        room.peers.insert(
            id.clone(),
            Peer {
                info: PeerInfo {
                    id: id.clone(),
                    name: name.unwrap_or_default().trim().into(),
                    online: false,
                },
                token: Uuid::new_v4().simple().to_string(),
                connection: String::new(),
                tx: None,
                last_seq: 0,
                seen: Instant::now(),
            },
        );
        id
    };
    let peer = room.peers.get_mut(&id).expect("inserted member");
    if peer.tx.is_some() {
        emit(peer, json!({"type":"replaced"}));
    }
    peer.tx = Some(tx);
    peer.connection = connection.into();
    peer.info.online = true;
    peer.seen = Instant::now();
    let welcome = json!({"type":"welcome","id":id,"code":code,"room_name":room.name,"resume_token":peer.token,"next_seq":peer.last_seq+1});
    // Replay only this member's messages. Stable sequence numbers prevent duplicate sends.
    for d in &room.messages {
        if d.sender == id {
            emit(peer, d.event.clone());
            emit(peer, d.status());
        } else if d.recipients.contains_key(&id) {
            emit(peer, d.event.clone());
        }
    }
    room.touched = Instant::now();
    presence(room);
    Ok((code, id, welcome))
}
async fn relay(
    hub: &Hub,
    code: &str,
    id: &str,
    connection: &str,
    command: Client,
) -> Option<Value> {
    let mut rooms = hub.rooms.lock().await;
    let room = rooms.get_mut(code)?;
    let peer = room.peers.get(id)?;
    if peer.connection != connection {
        return Some(error("Conexão substituída."));
    }
    let is_ping = matches!(&command, Client::Ping { .. });
    match command {
        Client::Chat {
            text,
            to,
            client_seq,
        } => {
            let seq = client_seq.unwrap_or(peer.last_seq + 1);
            let rejected = |message| {
                let mut value = error(message);
                value["client_seq"] = json!(seq);
                Some(value)
            };
            if seq == 0 || seq > 9_007_199_254_740_990 {
                return rejected("Sequência inválida.");
            }
            if seq <= peer.last_seq {
                if let Some(d) = room
                    .messages
                    .iter()
                    .find(|d| d.sender == id && d.seq == seq)
                {
                    emit(peer, d.event.clone());
                    return Some(d.status());
                }
                return Some(
                    json!({"type":"expired","client_seq":seq,"message":"Mensagem antiga fora da janela de recuperação. Não foi reenviada."}),
                );
            }
            if text.trim().is_empty() || text.chars().count() > 4000 {
                return rejected("A mensagem deve ter de 1 a 4000 caracteres.");
            }
            if let Some(target) = &to {
                if target == id || !room.peers.contains_key(target) {
                    return rejected("O destinatário não está conectado.");
                }
            }
            let recipients: BTreeMap<_, _> = room
                .peers
                .keys()
                .filter(|target| target.as_str() != id && to.as_ref().is_none_or(|t| t == *target))
                .map(|t| (t.clone(), 0))
                .collect();
            let event = json!({"type":"chat","id":Uuid::new_v4().simple().to_string(),"from":id,"name":peer.info.name,"text":text.trim(),"to":to,"client_seq":seq,"forwarded":recipients.len()});
            for target in recipients.keys() {
                if let Some(p) = room.peers.get(target) {
                    emit(p, event.clone());
                }
            }
            let delivery = Delivery {
                event: event.clone(),
                sender: id.into(),
                seq,
                recipients,
            };
            emit(peer, delivery.status());
            room.messages.push_back(delivery);
            while room.messages.len() > 256 {
                room.messages.pop_front();
            }
            room.peers.get_mut(id)?.last_seq = seq;
            room.touched = Instant::now();
            Some(event)
        }
        Client::Receipt {
            id: message_id,
            read,
        } => {
            let d = room
                .messages
                .iter_mut()
                .find(|d| d.event["id"] == message_id)?;
            let level = d.recipients.get_mut(id)?;
            *level = (*level).max(if read { 2 } else { 1 });
            if let Some(sender) = room.peers.get(&d.sender) {
                emit(sender, d.status());
            }
            None
        }
        Client::Ping { to, nonce } | Client::Pong { to, nonce } => {
            if nonce.is_empty() || nonce.len() > 64 {
                return Some(error("Identificador inválido."));
            }
            if let Some(target) = room.peers.get(&to).filter(|p| p.info.online && to != id) {
                emit(
                    target,
                    json!({"type":if is_ping{"ping"}else{"pong"},"from":id,"nonce":nonce}),
                );
                None
            } else {
                Some(error("O destinatário não está conectado."))
            }
        }
        Client::Heartbeat => Some(json!({"type":"heartbeat"})),
        _ => Some(error("Você já está em uma rede.")),
    }
}
