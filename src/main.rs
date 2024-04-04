// #![windows_subsystem = "windows"]
mod json_obj;
mod obj;
mod helper;

use lazy_static::lazy_static;
use tokio::sync::{Mutex, Notify};
use obj::*;
use helper::*;
use json_obj::*;
use tokio_tungstenite::{connect_async_tls_with_config, Connector};
use futures_util::StreamExt as _;



lazy_static! {
    static ref LOCKFILE_EXISTS: Mutex<bool> = Mutex::new(false);
    static ref VALORANT_IS_OPEN: Notify = Notify::new();
}

#[derive(Debug, Clone)]
struct Event {
    name: String,
    bytes: Option<Vec<u8>>,
    string: Option<String>
}



async fn read_lockfile() -> Lockfile {
    let logfile_path = logfile_path();
    let data = tokio::fs::read_to_string(logfile_path).await.unwrap();
    Lockfile::from_string(data)
}

async fn start_connect(event_tx: tokio::sync::mpsc::UnboundedSender<Event>) {
    println!("starting connect");
    let lock_data = read_lockfile().await;
    println!("connecting to wss url: {}", lock_data.to_wss_url().uri());
    let mut tls_cf_builder = native_tls::TlsConnector::builder();
    tls_cf_builder.danger_accept_invalid_certs(true);
    let tls_cf = tls_cf_builder.build().unwrap();
    let tls_cf_cnt = Connector::NativeTls(tls_cf);
    let raw_ws = connect_async_tls_with_config(lock_data.to_wss_url(), None, true, Some(tls_cf_cnt)).await;
    if raw_ws.is_err() {
        let _ = event_tx.send(Event {
            name: "riotclient_disconect".to_string(),
            bytes: None,
            string: None,
        });
        return;
    }
    let (mut ws_stream, _) = raw_ws.unwrap();
    println!("connected to wss");
    tokio::spawn(async move {
        loop {
            let d = ws_stream.next().await;
            println!("got data: {:?}", d);
            if d.is_none() {
                break;
            }
        }
        let _ = event_tx.send(Event {
            name: "riotclient_disconect".to_string(),
            bytes: None,
            string: None
        });
    });
    let client = local_client(&lock_data);
    let res = client.get(lock_data.to_url("/entitlements/v1/token")).send().await.unwrap();
    let tkobj: TokenResponse = res.json().await.unwrap();
    println!("{:?}", tkobj);
}

#[tokio::main]
async fn main() {
    let (event_tx, mut event_rx) = tokio::sync::mpsc::unbounded_channel();
    let cl_event_tx = event_tx.clone();
    tokio::spawn(async move {
        loop {
            let raw_data = event_rx.recv().await;
            if raw_data.is_none() {
                return;
            }
            let data: Event = raw_data.unwrap();
            match data.name.as_str() {
                "lockfile_exists" => {
                    *LOCKFILE_EXISTS.lock().await = true;
                    tokio::spawn(start_connect(cl_event_tx.clone()));
                },
                "riotclient_disconect" => {
                    *LOCKFILE_EXISTS.lock().await = false;
                    VALORANT_IS_OPEN.notify_waiters();
                },
                _ => { println!("{}: {:?}: {:?}", data.name, data.bytes, data.string); }
            }
        }
    });
    let cl_event_tx = event_tx.clone();
    tokio::spawn(async move {
        let logfile_path = logfile_path();
        loop {
            let path_exists = tokio::fs::try_exists(logfile_path.clone()).await;
            if path_exists.unwrap_or(false) {
                let _ = cl_event_tx.send(Event {
                    name: "lockfile_exists".to_string(),
                    bytes: None,
                    string: None
                });
            }
            tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
            println!("waiting for lockfile");
            if *LOCKFILE_EXISTS.lock().await {
                VALORANT_IS_OPEN.notified().await;
            }
        }
    });
    loop {}
}