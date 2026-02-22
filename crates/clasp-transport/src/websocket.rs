//! WebSocket transport implementation

use async_trait::async_trait;
use bytes::Bytes;
use futures_util::{SinkExt, StreamExt};
use parking_lot::Mutex;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::sync::mpsc;
use tokio::io::AsyncWriteExt;
use tokio_tungstenite::{
    connect_async,
    tungstenite::{
        handshake::{
            client::generate_key,
            server::{Request as HsRequest, Response as HsResponse},
        },
        http::Request,
        protocol::Message as WsMessage,
    },
};
use tracing::{debug, error, info, warn};

use crate::error::{Result, TransportError};
use crate::traits::{
    Transport, TransportEvent, TransportReceiver, TransportSender, TransportServer,
};

use clasp_core::WS_SUBPROTOCOL;

/// Default channel buffer size for WebSocket connections
/// Larger buffers help prevent message drops under load
pub const DEFAULT_CHANNEL_BUFFER_SIZE: usize = 1000;

/// WebSocket configuration
#[derive(Debug, Clone)]
pub struct WebSocketConfig {
    /// Subprotocol to use
    pub subprotocol: String,
    /// Maximum message size
    pub max_message_size: usize,
    /// Ping interval in seconds
    pub ping_interval: u64,
    /// Channel buffer size for send/receive queues
    pub channel_buffer_size: usize,
}

impl Default for WebSocketConfig {
    fn default() -> Self {
        Self {
            subprotocol: WS_SUBPROTOCOL.to_string(),
            max_message_size: 64 * 1024, // 64KB
            ping_interval: 30,
            channel_buffer_size: DEFAULT_CHANNEL_BUFFER_SIZE,
        }
    }
}

/// WebSocket transport
pub struct WebSocketTransport {
    #[allow(dead_code)]
    config: WebSocketConfig,
}

impl WebSocketTransport {
    pub fn new() -> Self {
        Self {
            config: WebSocketConfig::default(),
        }
    }

    pub fn with_config(config: WebSocketConfig) -> Self {
        Self { config }
    }
}

impl Default for WebSocketTransport {
    fn default() -> Self {
        Self::new()
    }
}

/// WebSocket sender
pub struct WebSocketSender {
    tx: mpsc::Sender<WsMessage>,
    connected: Arc<Mutex<bool>>,
}

#[async_trait]
impl TransportSender for WebSocketSender {
    async fn send(&self, data: Bytes) -> Result<()> {
        if !self.is_connected() {
            return Err(TransportError::NotConnected);
        }

        self.tx
            .send(WsMessage::Binary(data.to_vec()))
            .await
            .map_err(|e| TransportError::SendFailed(e.to_string()))
    }

    fn try_send(&self, data: Bytes) -> Result<()> {
        if !self.is_connected() {
            return Err(TransportError::NotConnected);
        }

        self.tx
            .try_send(WsMessage::Binary(data.to_vec()))
            .map_err(|e| match e {
                mpsc::error::TrySendError::Full(_) => TransportError::BufferFull,
                mpsc::error::TrySendError::Closed(_) => TransportError::ConnectionClosed,
            })
    }

    fn is_connected(&self) -> bool {
        *self.connected.lock()
    }

    async fn close(&self) -> Result<()> {
        let _ = self.tx.send(WsMessage::Close(None)).await;
        *self.connected.lock() = false;
        Ok(())
    }
}

/// WebSocket receiver
pub struct WebSocketReceiver {
    rx: mpsc::Receiver<TransportEvent>,
}

#[async_trait]
impl TransportReceiver for WebSocketReceiver {
    async fn recv(&mut self) -> Option<TransportEvent> {
        self.rx.recv().await
    }
}

#[async_trait]
impl Transport for WebSocketTransport {
    type Sender = WebSocketSender;
    type Receiver = WebSocketReceiver;

    async fn connect(url: &str) -> Result<(Self::Sender, Self::Receiver)> {
        info!("Connecting to WebSocket: {}", url);

        // Parse the URL to extract host for the Host header
        let parsed_url =
            url::Url::parse(url).map_err(|e| TransportError::InvalidUrl(e.to_string()))?;

        let host = parsed_url
            .host_str()
            .ok_or_else(|| TransportError::InvalidUrl("Missing host in URL".to_string()))?;

        let host_header = if let Some(port) = parsed_url.port() {
            format!("{}:{}", host, port)
        } else {
            host.to_string()
        };

        // Build a complete WebSocket upgrade request with all required headers
        let ws_key = generate_key();
        let request = Request::builder()
            .method("GET")
            .uri(url)
            .header("Host", &host_header)
            .header("Upgrade", "websocket")
            .header("Connection", "Upgrade")
            .header("Sec-WebSocket-Key", &ws_key)
            .header("Sec-WebSocket-Version", "13")
            .header("Sec-WebSocket-Protocol", WS_SUBPROTOCOL)
            .body(())
            .map_err(|e| TransportError::InvalidUrl(e.to_string()))?;

        // Connect
        let (ws_stream, response) = connect_async(request)
            .await
            .map_err(|e| TransportError::ConnectionFailed(e.to_string()))?;

        debug!("WebSocket connected, response: {:?}", response.status());

        // Check subprotocol
        if let Some(protocol) = response.headers().get("Sec-WebSocket-Protocol") {
            debug!("Server subprotocol: {:?}", protocol);
        }

        // Split the WebSocket stream
        let (write, read) = ws_stream.split();

        // Create channels with larger buffers for better load handling
        let (send_tx, mut send_rx) = mpsc::channel::<WsMessage>(DEFAULT_CHANNEL_BUFFER_SIZE);
        let (event_tx, event_rx) = mpsc::channel::<TransportEvent>(DEFAULT_CHANNEL_BUFFER_SIZE);

        let connected = Arc::new(Mutex::new(true));
        let connected_write = connected.clone();
        let connected_read = connected.clone();

        // Spawn writer task
        tokio::spawn(async move {
            let mut write = write;
            while let Some(msg) = send_rx.recv().await {
                if let Err(e) = write.send(msg).await {
                    error!("WebSocket write error: {}", e);
                    break;
                }
            }
            *connected_write.lock() = false;
        });

        // Spawn reader task
        let event_tx_clone = event_tx.clone();
        tokio::spawn(async move {
            let mut read = read;

            // Send connected event
            let _ = event_tx_clone.send(TransportEvent::Connected).await;

            while let Some(result) = read.next().await {
                match result {
                    Ok(msg) => {
                        match msg {
                            WsMessage::Binary(data) => {
                                let _ = event_tx_clone
                                    .send(TransportEvent::Data(Bytes::from(data)))
                                    .await;
                            }
                            WsMessage::Text(text) => {
                                // Convert text to bytes (shouldn't happen in Clasp)
                                warn!("Received text message, converting to bytes");
                                let _ = event_tx_clone
                                    .send(TransportEvent::Data(Bytes::from(text)))
                                    .await;
                            }
                            WsMessage::Ping(data) => {
                                debug!("Received ping");
                                // Pong is handled automatically by tungstenite
                                let _ = data;
                            }
                            WsMessage::Pong(_) => {
                                debug!("Received pong");
                            }
                            WsMessage::Close(frame) => {
                                let reason = frame.map(|f| f.reason.to_string());
                                info!("WebSocket closed: {:?}", reason);
                                let _ = event_tx_clone
                                    .send(TransportEvent::Disconnected { reason })
                                    .await;
                                break;
                            }
                            WsMessage::Frame(_) => {
                                // Raw frame, ignore
                            }
                        }
                    }
                    Err(e) => {
                        error!("WebSocket read error: {}", e);
                        let _ = event_tx_clone
                            .send(TransportEvent::Error(e.to_string()))
                            .await;
                        let _ = event_tx_clone
                            .send(TransportEvent::Disconnected {
                                reason: Some(e.to_string()),
                            })
                            .await;
                        break;
                    }
                }
            }

            *connected_read.lock() = false;
        });

        let sender = WebSocketSender {
            tx: send_tx,
            connected,
        };

        let receiver = WebSocketReceiver { rx: event_rx };

        Ok((sender, receiver))
    }

    fn local_addr(&self) -> Option<SocketAddr> {
        None
    }

    fn remote_addr(&self) -> Option<SocketAddr> {
        None
    }
}

/// WebSocket server
pub struct WebSocketServer {
    listener: tokio::net::TcpListener,
    config: WebSocketConfig,
}

impl WebSocketServer {
    pub async fn bind(addr: &str) -> Result<Self> {
        let listener = tokio::net::TcpListener::bind(addr)
            .await
            .map_err(|e| TransportError::ConnectionFailed(e.to_string()))?;

        info!("WebSocket server listening on {}", addr);

        Ok(Self {
            listener,
            config: WebSocketConfig::default(),
        })
    }

    pub fn with_config(mut self, config: WebSocketConfig) -> Self {
        self.config = config;
        self
    }
}

#[async_trait]
impl TransportServer for WebSocketServer {
    type Sender = WebSocketSender;
    type Receiver = WebSocketReceiver;

    async fn accept(&mut self) -> Result<(Self::Sender, Self::Receiver, SocketAddr)> {
        let (stream, addr) = loop {
            let (mut stream, addr) = self
                .listener
                .accept()
                .await
                .map_err(|e| TransportError::ConnectionFailed(e.to_string()))?;

            // Peek at incoming bytes to detect plain HTTP health checks.
            // App Platform (and other load balancers) send GET /healthz as a
            // plain HTTP request — not a WebSocket upgrade. Without this
            // intercept, tungstenite rejects them as bad handshakes.
            let mut peek_buf = [0u8; 512];
            match stream.peek(&mut peek_buf).await {
                Ok(n) if n > 0 => {
                    if let Ok(text) = std::str::from_utf8(&peek_buf[..n]) {
                        let is_health = text.starts_with("GET /healthz");
                        let is_ws_upgrade = text
                            .to_ascii_lowercase()
                            .contains("upgrade: websocket");
                        if is_health && !is_ws_upgrade {
                            debug!("Health check from {}, responding 200", addr);
                            let resp = "HTTP/1.1 200 OK\r\n\
                                        Content-Type: text/plain\r\n\
                                        Content-Length: 3\r\n\
                                        Connection: close\r\n\r\nok\n";
                            let _ = stream.try_write(resp.as_bytes());
                            let _ = stream.shutdown().await;
                            continue;
                        }
                    }
                }
                _ => {}
            }

            break (stream, addr);
        };

        debug!("Accepted TCP connection from {}", addr);

        // Upgrade to WebSocket with subprotocol negotiation
        let subprotocol = self.config.subprotocol.clone();
        let ws_stream = tokio_tungstenite::accept_hdr_async(
            stream,
            |req: &HsRequest, mut response: HsResponse| {
                // Check if client requested our subprotocol
                if let Some(protocols) = req.headers().get("Sec-WebSocket-Protocol") {
                    if let Ok(protocols_str) = protocols.to_str() {
                        // Client may request multiple protocols, comma-separated
                        let requested: Vec<&str> =
                            protocols_str.split(',').map(|s| s.trim()).collect();
                        if requested.contains(&subprotocol.as_str()) {
                            // Add our subprotocol to the response
                            response
                                .headers_mut()
                                .insert("Sec-WebSocket-Protocol", subprotocol.parse().unwrap());
                        }
                    }
                }
                Ok(response)
            },
        )
        .await
        .map_err(|e| TransportError::ConnectionFailed(e.to_string()))?;

        info!("WebSocket client connected from {}", addr);

        // Split the stream
        let (write, read) = ws_stream.split();

        // Create channels with configurable buffer size for better load handling
        let buffer_size = self.config.channel_buffer_size;
        let (send_tx, mut send_rx) = mpsc::channel::<WsMessage>(buffer_size);
        let (event_tx, event_rx) = mpsc::channel::<TransportEvent>(buffer_size);

        let connected = Arc::new(Mutex::new(true));
        let connected_write = connected.clone();
        let connected_read = connected.clone();

        // Spawn writer task
        tokio::spawn(async move {
            let mut write = write;
            while let Some(msg) = send_rx.recv().await {
                if let Err(e) = write.send(msg).await {
                    error!("WebSocket write error: {}", e);
                    break;
                }
            }
            *connected_write.lock() = false;
        });

        // Spawn reader task
        let event_tx_clone = event_tx.clone();
        tokio::spawn(async move {
            let mut read = read;

            let _ = event_tx_clone.send(TransportEvent::Connected).await;

            while let Some(result) = read.next().await {
                match result {
                    Ok(msg) => match msg {
                        WsMessage::Binary(data) => {
                            let _ = event_tx_clone
                                .send(TransportEvent::Data(Bytes::from(data)))
                                .await;
                        }
                        WsMessage::Close(frame) => {
                            let reason = frame.map(|f| f.reason.to_string());
                            let _ = event_tx_clone
                                .send(TransportEvent::Disconnected { reason })
                                .await;
                            break;
                        }
                        _ => {}
                    },
                    Err(e) => {
                        let _ = event_tx_clone
                            .send(TransportEvent::Disconnected {
                                reason: Some(e.to_string()),
                            })
                            .await;
                        break;
                    }
                }
            }

            *connected_read.lock() = false;
        });

        let sender = WebSocketSender {
            tx: send_tx,
            connected,
        };

        let receiver = WebSocketReceiver { rx: event_rx };

        Ok((sender, receiver, addr))
    }

    fn local_addr(&self) -> Result<SocketAddr> {
        self.listener.local_addr().map_err(TransportError::Io)
    }

    async fn close(&self) -> Result<()> {
        // TCP listener doesn't need explicit close
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_websocket_config() {
        let config = WebSocketConfig::default();
        assert_eq!(config.subprotocol, "clasp");
    }
}
