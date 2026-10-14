//! Local-user storage: encrypted values, no remote access and no cleartext fallback.
use axum::{
    extract::{ConnectInfo, Query},
    http::{HeaderMap, StatusCode},
    response::{IntoResponse, Response},
    Json,
};
use rusqlite::{params, Connection};
use serde::Deserialize;
use serde_json::{json, Value};
use std::{net::SocketAddr, path::PathBuf};

#[derive(Deserialize)]
pub struct Request {
    #[serde(default)]
    namespace: String,
    #[serde(default)]
    key: String,
    #[serde(default)]
    value: Value,
    #[serde(default)]
    action: String,
    #[serde(default)]
    offset: u32,
}
fn valid_key(s: &str) -> bool {
    !s.is_empty()
        && s.len() <= 160
        && s.bytes()
            .all(|b| b.is_ascii_alphanumeric() || b"-_:".contains(&b))
}
fn location() -> Result<PathBuf, String> {
    let root = std::env::var_os("LOCALAPPDATA")
        .ok_or("Armazenamento protegido disponível no Windows local.")?;
    Ok(PathBuf::from(root).join("NetLab").join("history-v1.sqlite"))
}
fn open() -> Result<Connection, String> {
    let file = location()?;
    std::fs::create_dir_all(file.parent().unwrap())
        .map_err(|_| "Não foi possível criar o armazenamento.")?;
    let db = Connection::open(file).map_err(|_| "Não foi possível abrir o histórico.")?;
    db.busy_timeout(std::time::Duration::from_secs(5))
        .map_err(|_| "Histórico ocupado")?;
    db.execute_batch("PRAGMA secure_delete=ON; CREATE TABLE IF NOT EXISTS records(n INTEGER PRIMARY KEY AUTOINCREMENT, namespace TEXT NOT NULL, k TEXT NOT NULL, value BLOB NOT NULL, UNIQUE(namespace,k));").map_err(|_|"Falha ao preparar o histórico.")?;
    Ok(db)
}
fn execute(db: &Connection, r: &Request) -> Result<Value, String> {
    if r.action == "batch" {
        let entries = r
            .value
            .as_array()
            .filter(|v| !v.is_empty() && v.len() <= 16)
            .ok_or("Lote inválido")?;
        let transaction = db
            .unchecked_transaction()
            .map_err(|_| "Histórico ocupado")?;
        for entry in entries {
            let item: Request =
                serde_json::from_value(entry.clone()).map_err(|_| "Registro inválido")?;
            if !matches!(item.action.as_str(), "put" | "delete" | "clear") {
                return Err("Ação inválida no lote".into());
            }
            execute(&transaction, &item)?;
        }
        transaction
            .commit()
            .map_err(|_| "Não foi possível salvar a transação")?;
        return Ok(json!({"saved":true}));
    }
    if !valid_key(&r.namespace) || (!r.key.is_empty() && !valid_key(&r.key)) {
        return Err("Identificador de histórico inválido.".into());
    }
    use sha2::{Digest, Sha256};
    let namespace = format!(
        "{:x}",
        Sha256::digest(format!("namespace:{}", r.namespace).as_bytes())
    );
    let key = format!(
        "{:x}",
        Sha256::digest(format!("key:{}:{}", r.namespace, r.key).as_bytes())
    );
    match r.action.as_str() {
        "get" => {
            use rusqlite::OptionalExtension;
            let raw: Option<Vec<u8>> = db
                .query_row(
                    "SELECT value FROM records WHERE namespace=?1 AND k=?2",
                    params![namespace, key],
                    |row| row.get(0),
                )
                .optional()
                .map_err(|_| "Falha ao ler registro")?;
            let record = match raw {
                Some(bytes) => serde_json::from_slice::<Value>(&protect(&bytes, false)?)
                    .map_err(|_| "Registro inválido")?,
                None => Value::Null,
            };
            Ok(json!({"record":record}))
        }
        "put" => {
            if r.key.is_empty() {
                return Err("Registro sem identificador".into());
            }
            let text = serde_json::to_vec(&json!({"key":r.key,"value":r.value}))
                .map_err(|_| "Dados inválidos")?;
            if text.len() > 65536 {
                return Err("Registro muito grande".into());
            }
            let encrypted = protect(&text, true)?;
            db.execute("INSERT INTO records(namespace,k,value) VALUES(?1,?2,?3) ON CONFLICT(namespace,k) DO UPDATE SET value=excluded.value",params![namespace,key,encrypted]).map_err(|_|"Não foi possível salvar. Confira o espaço livre.")?;
            Ok(json!({"saved":true}))
        }
        "delete" => {
            db.execute(
                "DELETE FROM records WHERE namespace=?1 AND k=?2",
                params![namespace, key],
            )
            .map_err(|_| "Falha ao apagar registro")?;
            Ok(json!({"deleted":true}))
        }
        "clear" => {
            db.execute("DELETE FROM records WHERE namespace=?1", params![namespace])
                .map_err(|_| "Falha ao apagar conversa")?;
            Ok(json!({"deleted":true}))
        }
        "" | "list" => {
            let mut statement=db.prepare("SELECT k,value FROM records WHERE namespace=?1 ORDER BY n DESC LIMIT 200 OFFSET ?2").map_err(|_|"Falha ao ler histórico")?;
            let rows = statement
                .query_map(params![namespace, r.offset], |row| {
                    Ok((row.get::<_, String>(0)?, row.get::<_, Vec<u8>>(1)?))
                })
                .map_err(|_| "Falha ao ler histórico")?;
            let mut values = Vec::new();
            for row in rows {
                let (_k, bytes) = row.map_err(|_| "Registro inválido")?;
                let clear = protect(&bytes, false)?;
                let value: Value = serde_json::from_slice(&clear)
                    .map_err(|_| "Registro corrompido; arquivos preservados")?;
                values.push(value);
            }
            let count: i64 = db
                .query_row(
                    "SELECT count(*) FROM records WHERE namespace=?1",
                    params![namespace],
                    |row| row.get(0),
                )
                .map_err(|_| "Falha ao contar histórico")?;
            Ok(json!({"records":values,"count":count,"protectedBy":"Windows DPAPI"}))
        }
        _ => Err("Ação de histórico inválida".into()),
    }
}
async fn dispatch(remote: SocketAddr, headers: HeaderMap, r: Request) -> Response {
    if !crate::bluetooth::allowed(remote, &headers) {
        return StatusCode::FORBIDDEN.into_response();
    }
    match tokio::task::spawn_blocking(move||{let db=open()?;execute(&db,&r)}).await{
  Ok(Ok(value))=>Json(value).into_response(),
  _=>(StatusCode::SERVICE_UNAVAILABLE,Json(json!({"error":"Histórico protegido indisponível. Use localhost no Windows e confira o espaço livre. Os arquivos existentes foram preservados."}))).into_response()
 }
}
pub async fn read(
    ConnectInfo(remote): ConnectInfo<SocketAddr>,
    headers: HeaderMap,
    Query(mut r): Query<Request>,
) -> Response {
    r.action = "list".into();
    dispatch(remote, headers, r).await
}
pub async fn write(
    ConnectInfo(remote): ConnectInfo<SocketAddr>,
    headers: HeaderMap,
    Json(r): Json<Request>,
) -> Response {
    dispatch(remote, headers, r).await
}

#[cfg(windows)]
fn protect(bytes: &[u8], encrypt: bool) -> Result<Vec<u8>, String> {
    use windows_sys::Win32::{Foundation::LocalFree, Security::Cryptography::*};
    unsafe {
        let input = CRYPT_INTEGER_BLOB {
            cbData: bytes.len() as u32,
            pbData: bytes.as_ptr() as *mut u8,
        };
        let mut output = CRYPT_INTEGER_BLOB::default();
        let ok = if encrypt {
            CryptProtectData(
                &input,
                std::ptr::null(),
                std::ptr::null(),
                std::ptr::null_mut(),
                std::ptr::null(),
                CRYPTPROTECT_UI_FORBIDDEN,
                &mut output,
            )
        } else {
            CryptUnprotectData(
                &input,
                std::ptr::null_mut(),
                std::ptr::null(),
                std::ptr::null_mut(),
                std::ptr::null(),
                CRYPTPROTECT_UI_FORBIDDEN,
                &mut output,
            )
        };
        if ok == 0 {
            return Err("Proteção Windows indisponível; arquivos preservados".into());
        }
        let result = std::slice::from_raw_parts(output.pbData, output.cbData as usize).to_vec();
        LocalFree(output.pbData.cast());
        Ok(result)
    }
}
#[cfg(not(windows))]
fn protect(_: &[u8], _: bool) -> Result<Vec<u8>, String> {
    Err("DPAPI exige Windows".into())
}

#[cfg(all(test, windows))]
mod tests {
    use super::*;
    #[test]
    fn encrypted_records_survive_reopen_and_clear() {
        let path =
            std::env::temp_dir().join(format!("netlab-test-{}.sqlite", uuid::Uuid::new_v4()));
        let db = Connection::open(&path).unwrap();
        db.execute_batch("CREATE TABLE records(n INTEGER PRIMARY KEY AUTOINCREMENT,namespace TEXT,k TEXT,value BLOB,UNIQUE(namespace,k));").unwrap();
        let mut r = Request {
            namespace: "test".into(),
            key: "message-1".into(),
            value: json!({"text":"private sentinel"}),
            action: "put".into(),
            offset: 0,
        };
        execute(&db, &r).unwrap();
        drop(db);
        assert!(!std::fs::read(&path)
            .unwrap()
            .windows(16)
            .any(|v| v == b"private sentinel"));
        let db = Connection::open(&path).unwrap();
        r.action = "list".into();
        assert_eq!(
            execute(&db, &r).unwrap()["records"][0]["value"]["text"],
            "private sentinel"
        );
        r.action = "clear".into();
        execute(&db, &r).unwrap();
        r.action = "list".into();
        assert_eq!(execute(&db, &r).unwrap()["count"], 0);
        drop(db);
        std::fs::remove_file(path).unwrap();
    }
    #[test]
    fn rejects_unsafe_record_identifiers() {
        assert!(!valid_key("../secret"));
        assert!(!valid_key(""));
        assert!(valid_key("bt-AA:BB:CC:DD:EE:FF"));
    }
    #[test]
    fn rejects_tampered_ciphertext() {
        let mut bytes = protect(b"private", true).unwrap();
        let i = bytes.len() - 1;
        bytes[i] ^= 1;
        assert!(protect(&bytes, false).is_err());
    }
    #[test]
    fn batch_rolls_back_and_get_is_scoped_to_namespace() {
        let db = Connection::open_in_memory().unwrap();
        db.execute_batch("CREATE TABLE records(n INTEGER PRIMARY KEY AUTOINCREMENT,namespace TEXT,k TEXT,value BLOB,UNIQUE(namespace,k));").unwrap();
        let request = |value: Value| serde_json::from_value::<Request>(value).unwrap();
        let batch = request(
            json!({"action":"batch","value":[{"action":"put","namespace":"one","key":"m1","value":{"text":"atomic"}},{"action":"put","namespace":"../invalid","key":"m2","value":{}}]}),
        );
        assert!(execute(&db, &batch).is_err());
        assert_eq!(
            execute(&db, &request(json!({"namespace":"one"}))).unwrap()["count"],
            0
        );
        execute(
            &db,
            &request(
                json!({"action":"put","namespace":"one","key":"m1","value":{"text":"atomic"}}),
            ),
        )
        .unwrap();
        assert_eq!(
            execute(
                &db,
                &request(json!({"action":"get","namespace":"one","key":"m1"}))
            )
            .unwrap()["record"]["value"]["text"],
            "atomic"
        );
        assert!(execute(
            &db,
            &request(json!({"action":"get","namespace":"two","key":"m1"}))
        )
        .unwrap()["record"]
            .is_null());
    }
    #[tokio::test]
    async fn denies_remote_clients_and_foreign_origins() {
        let request = || serde_json::from_value::<Request>(json!({"namespace":"test"})).unwrap();
        let mut headers = HeaderMap::new();
        headers.insert("host", "localhost:8787".parse().unwrap());
        assert_eq!(
            dispatch(
                "192.0.2.1:4321".parse().unwrap(),
                headers.clone(),
                request()
            )
            .await
            .status(),
            StatusCode::FORBIDDEN
        );
        headers.insert("origin", "https://untrusted.example".parse().unwrap());
        assert_eq!(
            dispatch("127.0.0.1:4321".parse().unwrap(), headers, request())
                .await
                .status(),
            StatusCode::FORBIDDEN
        );
    }
}
