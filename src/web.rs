use base64;
use sha1::{Digest, Sha1};
use std::collections::HashMap;
use std::env;
use std::fs;
use std::io::{Read, Write};
use std::net::TcpListener;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::mpsc::channel;
use std::sync::mpsc::Receiver;
use std::sync::mpsc::Sender;
use std::sync::{Arc, Mutex};
use std::thread;

static NEXT_ID: AtomicUsize = AtomicUsize::new(1);
type WebSocketConnections = Arc<Mutex<HashMap<usize, std::net::TcpStream>>>;

pub enum WebSignal {
    GetChain { client_id: usize },
    GetPeers { client_id: usize },
}

#[derive(Debug)]
pub struct WebServer {
    listener: TcpListener,
    connections: WebSocketConnections,
    signal_tx: Sender<WebSignal>,
}

impl WebServer {
    pub fn new(address: &str) -> (Arc<Self>, Receiver<WebSignal>) {
        let (tx, rx) = channel();
        let listener = TcpListener::bind(address).unwrap();
        let connections = Arc::new(Mutex::new(HashMap::new()));

        let server = Self {
            listener,
            connections,
            signal_tx: tx,
        };

        (Arc::new(server), rx)
    }

    pub fn run(self: &Arc<Self>) {
        println!(
            "Web server running on {}",
            self.listener.local_addr().unwrap()
        );

        let server_clone = Arc::clone(self);
        thread::spawn(move || {
            for stream in server_clone.listener.incoming() {
                let connections = Arc::clone(&server_clone.connections);
                let server = Arc::clone(&server_clone);
                thread::spawn(move || server.handle_request(stream.unwrap(), connections));
            }
        });
    }

    // New methods for WebSocket API
    pub fn broadcast_message(&self, message: &[u8]) {
        let connections = self.connections.lock().unwrap();
        for stream in connections.values() {
            if let Ok(mut stream) = stream.try_clone() {
                let _ = self.send_frame(&mut stream, message);
            }
        }
    }

    pub fn send_to_client(&self, client_id: usize, message: &[u8]) -> bool {
        let connections = self.connections.lock().unwrap();
        if let Some(stream) = connections.get(&client_id) {
            if let Ok(mut stream) = stream.try_clone() {
                self.send_frame(&mut stream, message);
                return true;
            }
        }
        false
    }

    pub fn get_active_connections(&self) -> Vec<usize> {
        self.connections.lock().unwrap().keys().cloned().collect()
    }

    fn handle_request(&self, mut stream: std::net::TcpStream, connections: WebSocketConnections) {
        let mut buffer = [0; 8192];
        stream.read(&mut buffer).unwrap();

        let request = String::from_utf8_lossy(&buffer[..]);

        if request.contains("Upgrade: websocket") {
            self.handle_websocket(&mut stream, &request, connections);
        } else {
            self.handle_http(&mut stream, &request);
        }
    }

    fn handle_websocket(
        &self,
        stream: &mut std::net::TcpStream,
        request: &str,
        connections: WebSocketConnections,
    ) {
        // Extract WebSocket key
        let key = request
            .lines()
            .find(|l| l.starts_with("Sec-WebSocket-Key:"))
            .unwrap()
            .split(": ")
            .nth(1)
            .unwrap()
            .trim();

        // Generate accept key
        let accept_key = self.generate_accept_key(key);

        // Send upgrade response
        let response = format!(
            "HTTP/1.1 101 Switching Protocols\r\n\
             Upgrade: websocket\r\n\
             Connection: Upgrade\r\n\
             Sec-WebSocket-Accept: {}\r\n\r\n",
            accept_key
        );

        stream.write(response.as_bytes()).unwrap();

        // Register the connection
        let id = NEXT_ID.fetch_add(1, Ordering::SeqCst);
        let stream_clone = stream.try_clone().unwrap();
        connections.lock().unwrap().insert(id, stream_clone);

        // Handle frames
        self.handle_websocket_frames(stream, id, connections);
    }

    fn handle_websocket_frames(
        &self,
        stream: &mut std::net::TcpStream,
        id: usize,
        connections: WebSocketConnections,
    ) {
        let mut buffer = [0; 8192];
        loop {
            match stream.read(&mut buffer) {
                Ok(n) if n > 0 => {
                    // Just keep the connection alive by reading messages
                    let _ = self.decode_frame(&buffer[..n]);
                    // Self::send_frame(stream, b"Hello");
                    let _ = self.signal_tx.send(WebSignal::GetChain { client_id: id });
                    let _ = self.signal_tx.send(WebSignal::GetPeers { client_id: id });
                }
                _ => {
                    // Remove connection when it's closed
                    connections.lock().unwrap().remove(&id);
                    break;
                }
            }
        }
    }

    fn get_content_type(&self, path: &str) -> &str {
        match path.split('.').last().unwrap_or("") {
            "txt" => "text/plain",
            "html" => "text/html",
            "js" => "application/javascript",
            "css" => "text/css",
            "svg" => "image/svg+xml",
            "json" => "application/json",
            "png" => "image/png",
            "jpg" | "jpeg" => "image/jpeg",
            "gif" => "image/gif",
            "ico" => "image/x-icon",
            "woff" => "font/woff",
            "woff2" => "font/woff2",
            "ttf" => "font/ttf",
            "webp" => "image/webp",
            _ => "application/octet-stream",
        }
    }

    fn handle_http(&self, stream: &mut std::net::TcpStream, request: &str) {
        let path = request
            .lines()
            .next()
            .unwrap()
            .split_whitespace()
            .nth(1)
            .expect("Invalid request");

        let current_dir = env::current_dir().unwrap();
        let dist_path = current_dir.join("web/dist");
        let file_path = if path == "/" {
            dist_path.join("index.html")
        } else {
            dist_path.join(&path[1..])
        };

        let response = match fs::read(&file_path) {
            Ok(content) => format!(
                "HTTP/1.1 200 OK\r\n\
                    Content-Type: {}\r\n\
                    Content-Length: {}\r\n\r\n",
                self.get_content_type(&file_path.to_string_lossy()),
                content.len()
            )
            .into_bytes()
            .into_iter()
            .chain(content)
            .collect::<Vec<u8>>(),
            Err(_) => format!(
                "HTTP/1.1 404 Not Found\r\n\
                    Content-Length: 9\r\n\r\n\
                    Not Found"
            )
            .into_bytes(),
        };

        stream.write(&response).unwrap();
    }

    fn generate_accept_key(&self, key: &str) -> String {
        let mut hasher = Sha1::new();
        hasher.update(format!("{}258EAFA5-E914-47DA-95CA-C5AB0DC85B11", key).as_bytes());
        base64::Engine::encode(
            &base64::engine::general_purpose::STANDARD,
            hasher.finalize(),
        )
    }

    fn decode_frame(&self, buffer: &[u8]) -> Option<Vec<u8>> {
        if buffer.len() < 2 {
            return None;
        }

        let masked = (buffer[1] & 0x80) != 0;
        let mut payload_len = (buffer[1] & 0x7F) as usize;

        let mut mask_offset = 2;
        if payload_len == 126 {
            payload_len = u16::from_be_bytes([buffer[2], buffer[3]]) as usize;
            mask_offset = 4;
        }

        if masked {
            let mask = &buffer[mask_offset..mask_offset + 4];
            let payload = &buffer[mask_offset + 4..mask_offset + 4 + payload_len];
            let mut unmasked = vec![0; payload_len];

            for i in 0..payload_len {
                unmasked[i] = payload[i] ^ mask[i % 4];
            }
            Some(unmasked)
        } else {
            None
        }
    }

    fn send_frame(&self, stream: &mut std::net::TcpStream, payload: &[u8]) {
        let mut frame = vec![0x81]; // FIN + Text frame

        if payload.len() < 126 {
            frame.push(payload.len() as u8);
        } else {
            frame.push(126);
            frame.extend_from_slice(&(payload.len() as u16).to_be_bytes());
        }

        frame.extend_from_slice(payload);
        stream.write(&frame).unwrap();
    }
}
