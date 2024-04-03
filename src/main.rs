// #![windows_subsystem = "windows"]
mod json_obj;
mod obj;
mod helper;

use lazy_static::lazy_static;
use tokio::{io::AsyncWriteExt as _, sync::Mutex};
use aio_event::AIOEvent;
use obj::*;
use helper::*;
use tokio_tungstenite::{connect_async_tls_with_config, tungstenite::Message, Connector};
use futures_util::{SinkExt as _, StreamExt as _};

use crate::json_obj::HelpResponse;


lazy_static! {
    static ref LOCKFILE_EXISTS: Mutex<bool> = Mutex::new(false);
    static ref VALORANT_IS_OPEN: Mutex<AIOEvent> = Mutex::new(AIOEvent::new());
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
    let (ws_w_tx, mut ws_w_rx) = tokio::sync::mpsc::unbounded_channel();
    println!("connecting to wss url: {}", lock_data.to_wss_url().uri());
    let mut tls_cf_builder = native_tls::TlsConnector::builder();
    tls_cf_builder.danger_accept_invalid_certs(true);
    let tls_cf = tls_cf_builder.build().unwrap();
    let tls_cf_cnt = Connector::NativeTls(tls_cf);
    let raw_ws = connect_async_tls_with_config(lock_data.to_wss_url(), None, true, Some(tls_cf_cnt)).await;
    println!("connected to wss");
    if raw_ws.is_err() {
        let _ = event_tx.send(Event {
            name: "valorant_disconect".to_string(),
            bytes: None,
            string: None,
        });
    }
    let (ws_stream, _) = raw_ws.unwrap();
    let (mut write_ws, mut read_ws) = ws_stream.split();
    tokio::spawn(async move {
        loop {
            let raw_data = ws_w_rx.recv().await;
            if raw_data.is_none() {
                return;
            }
            let data = raw_data.unwrap();
            write_ws.send(data).await.unwrap();
        }
    });
    tokio::spawn(async move {
        while let Some(Ok(msg)) = read_ws.next().await {
            let text = msg.into_text().unwrap();
            if text.is_empty() {
                continue;
            } else if text.contains("\"/social/v1/friends") {
                continue;
            }
            let _ = event_tx.send(Event {
                name: "ws_msg".to_string(),
                bytes: None,
                string: Some(text),
            });
        }
        let _ = event_tx.send(Event {
            name: "valorant_disconect".to_string(),
            bytes: None,
            string: None
        });
    });
    let client = reqwest::ClientBuilder::new().danger_accept_invalid_certs(true).build().unwrap();
    let res = client.get(lock_data.to_url("/help"))
        .header("Authorization", lock_data.auth_header())
        .send()
        .await.unwrap();
    let help_response: HelpResponse = res.json().await.unwrap();
    for (key, _) in help_response.events {
        let _ = ws_w_tx.send(Message::text(format!("[5, \"{}\"]", key)));
    }
}

#[tokio::main]
async fn main() {
    VALORANT_IS_OPEN.lock().await.set().await;
    let (event_tx, mut event_rx) = tokio::sync::mpsc::unbounded_channel();
    let cl_event_tx = event_tx.clone();
    tokio::spawn(async move {
        let mut time = 0;
        loop {
            let raw_data = event_rx.recv().await;
            if raw_data.is_none() {
                return;
            }
            let data: Event = raw_data.unwrap();
            match data.name.as_str() {
                "lockfile_exists" => {
                    *LOCKFILE_EXISTS.lock().await = true;
                    VALORANT_IS_OPEN.lock().await.clear().await;
                    tokio::spawn(start_connect(cl_event_tx.clone()));
                    println!("lockfile exists");
                },
                "valorant_disconect" => {
                    *LOCKFILE_EXISTS.lock().await = false;
                    VALORANT_IS_OPEN.lock().await.set().await;
                },
                "ws_msg" => {
                    println!("{}: {:?}: {:?}", data.name, data.bytes, data.string);
                    tokio::fs::File::create(format!("./test/{}.json", time)).await.unwrap().write_all(data.string.clone().unwrap().as_bytes()).await.unwrap();
                    time += 1;
                },
                _ => {}
            }
            println!("{}: {:?}: {:?}", data.name, data.bytes, data.string);
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
                VALORANT_IS_OPEN.lock().await.wait().await;
            }
        }
    });
    loop {}
}