use std::path::Path;
use prost::Message;
use std::io::{self, Read};

use crate::crowdllama_pb::*;

pub struct SocketListener;

impl SocketListener {
    pub fn new() -> Self {
        Self
    }

    pub async fn start(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        #[cfg(unix)]
        {
            self.start_unix().await
        }
        
        #[cfg(windows)]
        {
            self.start_windows().await
        }
    }

    #[cfg(unix)]
    async fn start_unix(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        use tokio::net::UnixListener;
        use tokio::io::AsyncReadExt;
        
        if Path::new(SOCKET_PATH).exists() {
            std::fs::remove_file(SOCKET_PATH)?;
        }
        let listener = UnixListener::bind(SOCKET_PATH)?;
        println!("Socket listener started on {}", SOCKET_PATH);

        // Run the accept loop directly here to keep the listener alive
        Self::listen_for_messages_unix(listener).await;
        Ok(())
    }

    #[cfg(windows)]
    async fn start_windows(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        println!("⚠️  Unix domain sockets are not supported on Windows");
        println!("📝 This is a stub implementation for Windows builds");
        println!("🔧 For production Windows builds, implement TCP or named pipes");
        
        // Return success to avoid breaking the build
        Ok(())
    }

    #[cfg(unix)]
    async fn listen_for_messages_unix(listener: tokio::net::UnixListener) {
        use tokio::io::AsyncReadExt;
        
        loop {
            match listener.accept().await {
                Ok((mut stream, _addr)) => {
                    println!("New connection accepted");
                    
                    loop {
                        // Read length prefix (4 bytes, big-endian)
                        let mut length_buf = [0u8; 4];
                        match stream.read_exact(&mut length_buf).await {
                            Ok(_) => {
                                let message_length = u32::from_be_bytes(length_buf) as usize;
                                println!("📏 Reading message of length: {} bytes", message_length);
                                
                                // Read the protobuf message data
                                let mut message_data = vec![0u8; message_length];
                                match stream.read_exact(&mut message_data).await {
                                    Ok(_) => {
                                        // Try to decode as protobuf
                                        match Self::decode_protobuf_message(&message_data) {
                                            Ok(message) => {
                                                println!("✅ Received protobuf message: {}", message);
                                            }
                                            Err(e) => {
                                                println!("❌ Failed to decode protobuf message: {}", e);
                                                // Show first few bytes for debugging
                                                let preview_len = std::cmp::min(message_data.len(), 50);
                                                println!("🔍 Message preview: {:?}", &message_data[..preview_len]);
                                            }
                                        }
                                    }
                                    Err(e) => {
                                        println!("Error reading message data: {}", e);
                                        break;
                                    }
                                }
                            }
                            Err(e) => {
                                if e.kind() == io::ErrorKind::UnexpectedEof {
                                    println!("Connection closed by client");
                                } else {
                                    println!("Error reading length prefix: {}", e);
                                }
                                break;
                            }
                        }
                    }
                }
                Err(e) => {
                    println!("Error accepting connection: {}", e);
                }
            }
        }
    }

    fn decode_protobuf_message(data: &[u8]) -> Result<String, Box<dyn std::error::Error>> {
        // Try to decode as BaseMessage first
        if let Ok(msg) = crate::crowdllama_pb_llama::BaseMessage::decode(data) {
            match msg.message {
                Some(crate::crowdllama_pb_llama::base_message::Message::GenerateRequest(req)) => {
                    return Ok(format!("🔄 GenerateRequest: model={}, prompt={}, stream={}", 
                        req.model, req.prompt, req.stream));
                }
                Some(crate::crowdllama_pb_llama::base_message::Message::GenerateResponse(resp)) => {
                    return Ok(format!("✅ GenerateResponse: model={}, response={}, done={}, worker_id={}", 
                        resp.model, resp.response, resp.done, resp.worker_id));
                }
                None => {
                    return Ok("📭 BaseMessage: empty message".to_string());
                }
            }
        }
        
        // Fallback to other message types if BaseMessage fails
        if let Ok(msg) = GenericMessage::decode(data) {
            return Ok(format!("📦 GenericMessage: id={}, type={}", msg.id, msg.r#type));
        }
        
        if let Ok(msg) = PingRequest::decode(data) {
            return Ok(format!("🏓 PingRequest: message={}", msg.message));
        }
        
        if let Ok(msg) = PingResponse::decode(data) {
            return Ok(format!("🏓 PingResponse: message={}, latency={}ms", msg.message, msg.latency_ms));
        }
        
        if let Ok(msg) = HealthCheckRequest::decode(data) {
            return Ok(format!("🏥 HealthCheckRequest: service={}", msg.service));
        }
        
        if let Ok(msg) = HealthCheckResponse::decode(data) {
            let status = Status::try_from(msg.status).unwrap_or(Status::Unspecified);
            return Ok(format!("🏥 HealthCheckResponse: service={}, status={:?}", msg.service, status));
        }
        
        if let Ok(msg) = StatusResponse::decode(data) {
            let status = Status::try_from(msg.status).unwrap_or(Status::Unspecified);
            return Ok(format!("📊 StatusResponse: status={:?}, message={}", status, msg.message));
        }
        
        if let Ok(msg) = Initialize::decode(data) {
            let mode = Mode::try_from(msg.mode).unwrap_or(Mode::Unspecified);
            return Ok(format!("🚀 Initialize: mode={:?}", mode));
        }
        
        if let Ok(msg) = NetworkStatus::decode(data) {
            let state = NetworkState::try_from(msg.state).unwrap_or(NetworkState::Unspecified);
            return Ok(format!("🌐 NetworkStatus: state={:?}, peer_count={}", state, msg.peer_count));
        }
        
        Err("Unknown message type".into())
    }
}

pub const SOCKET_PATH: &str = "/tmp/crowdllama.sock";

pub async fn start_socket_listener() {
    let mut socket_listener = SocketListener::new();
    if let Err(e) = socket_listener.start().await {
        eprintln!("Failed to start socket listener: {}", e);
    }
} 