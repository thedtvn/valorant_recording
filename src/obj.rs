use tokio_tungstenite::tungstenite::{http::Request as Request_tungstenite};
use base64::{engine::general_purpose::URL_SAFE, Engine as _};


#[derive(Debug, Default)]
pub struct Lockfile {
    pub name: String,
    pub pid: String,
    pub port: String,
    pub password: String,
    pub protocol: String,
}

impl Lockfile {
    pub fn from_string(s: String) -> Lockfile {
        let parts: Vec<&str> = s.split(':').collect();

        match parts.as_slice() {
            [name, pid, port, password, protocol, ..] => Lockfile {
                name: name.to_string(),
                pid: pid.to_string(),
                port: port.to_string(),
                password: password.to_string(),
                protocol: protocol.to_string(),
            },
            _ => Lockfile::default(),
        }
    }

    pub fn to_url(&self, path: &str) -> String {
        return format!("{}://riot:{}@localhost:{}{}", self.protocol, self.password, self.port, path);
    }

    pub fn auth_header(&self) -> String {
        let token = URL_SAFE.encode(format!("riot:{}", self.password));
        format!("Basic {}", token)
    }

    pub fn to_wss_url(&self) -> Request_tungstenite<()> {
        let protocol = if self.protocol == "http" { "ws" } else { "wss" };
        let url = format!("{}://riot:{}@localhost:{}/", protocol, self.password, self.port);
        let req = Request_tungstenite::builder()
        .method("GET")
        .uri(url)
        .header("Authorization", self.auth_header())
        .header("sec-websocket-key", "foo")
        .header("host", "server.example.com")
        .header("upgrade", "websocket")
        .header("connection", "upgrade")
        .header("sec-websocket-version", 13)
        .body(())
        .unwrap();
        return req;
    }
}