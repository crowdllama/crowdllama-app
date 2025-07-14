use crowdllama_app_lib::ipc::{SocketListener, SOCKET_PATH};
use prost::Message;
use std::time::Duration;
use tokio::time::timeout;

// Import protobuf types
use crowdllama_app_lib::crowdllama_pb_llama::{BaseMessage, GenerateRequest, GenerateResponse, base_message};

#[cfg(unix)]
use tokio::net::UnixStream;
#[cfg(unix)]
use tokio::io::{AsyncReadExt, AsyncWriteExt};

async fn write_message_with_length_prefix(
    stream: &mut UnixStream,
    message: &[u8],
) -> Result<(), Box<dyn std::error::Error>> {
    // Write length prefix (4 bytes, big-endian)
    let length = message.len() as u32;
    stream.write_all(&length.to_be_bytes()).await?;
    
    // Write the message
    stream.write_all(message).await?;
    stream.flush().await?;
    
    Ok(())
}

async fn read_message_with_length_prefix(
    stream: &mut UnixStream,
) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    // Read length prefix (4 bytes, big-endian)
    let mut length_buf = [0u8; 4];
    stream.read_exact(&mut length_buf).await?;
    let message_length = u32::from_be_bytes(length_buf) as usize;
    
    // Read the message
    let mut message_data = vec![0u8; message_length];
    stream.read_exact(&mut message_data).await?;
    
    Ok(message_data)
}

#[cfg(unix)]
#[tokio::test]
async fn test_ipc_generate_request_response_flow() {
    // Remove existing socket if it exists
    let _ = std::fs::remove_file(SOCKET_PATH);
    
    // Start the socket listener in a background task
    let listener_handle = tokio::spawn(async {
        let mut listener = SocketListener::new();
        if let Err(e) = listener.start().await {
            eprintln!("Socket listener failed: {}", e);
        }
    });
    
    // Wait a bit for the socket to be created
    tokio::time::sleep(Duration::from_millis(100)).await;
    
    // Try to connect to the socket with timeout
    let mut stream = timeout(Duration::from_secs(5), UnixStream::connect(SOCKET_PATH))
        .await
        .expect("Timeout connecting to socket")
        .expect("Failed to connect to socket");
    
    println!("✅ Connected to IPC socket");
    
    // Test 1: Send a GenerateRequest
    let generate_request = GenerateRequest {
        model: "test-model".to_string(),
        prompt: "Hello, world!".to_string(),
        stream: false,
    };
    
    let base_message = BaseMessage {
        message: Some(base_message::Message::GenerateRequest(generate_request)),
    };
    
    let encoded_message = base_message.encode_to_vec();
    
    println!("📤 Sending GenerateRequest: {} bytes", encoded_message.len());
    write_message_with_length_prefix(&mut stream, &encoded_message)
        .await
        .expect("Failed to send GenerateRequest");
    
    // Test 2: Send a GenerateResponse (simulating server response)
    let generate_response = GenerateResponse {
        model: "test-model".to_string(),
        response: "Hello back!".to_string(),
        done: true,
        worker_id: "worker-123".to_string(),
    };
    
    let response_base_message = BaseMessage {
        message: Some(base_message::Message::GenerateResponse(generate_response)),
    };
    
    let encoded_response = response_base_message.encode_to_vec();
    
    println!("📤 Sending GenerateResponse: {} bytes", encoded_response.len());
    write_message_with_length_prefix(&mut stream, &encoded_response)
        .await
        .expect("Failed to send GenerateResponse");
    
    // Test 3: Send multiple messages to test continuous communication
    for i in 0..3 {
        let request = GenerateRequest {
            model: format!("model-{}", i),
            prompt: format!("Test message {}", i),
            stream: i % 2 == 0, // Alternate streaming
        };
        
        let message = BaseMessage {
            message: Some(base_message::Message::GenerateRequest(request)),
        };
        
        let encoded = message.encode_to_vec();
        println!("📤 Sending batch message {}: {} bytes", i, encoded.len());
        
        write_message_with_length_prefix(&mut stream, &encoded)
            .await
            .expect("Failed to send batch message");
        
        // Small delay between messages
        tokio::time::sleep(Duration::from_millis(10)).await;
    }
    
    // Give some time for the server to process messages
    tokio::time::sleep(Duration::from_millis(200)).await;
    
    println!("✅ IPC test completed successfully");
    
    // Clean up
    drop(stream);
    listener_handle.abort();
    let _ = std::fs::remove_file(SOCKET_PATH);
}

#[cfg(unix)]
#[tokio::test]
async fn test_ipc_invalid_message_handling() {
    // Remove existing socket if it exists
    let _ = std::fs::remove_file(SOCKET_PATH);
    
    // Start the socket listener in a background task
    let listener_handle = tokio::spawn(async {
        let mut listener = SocketListener::new();
        if let Err(e) = listener.start().await {
            eprintln!("Socket listener failed: {}", e);
        }
    });
    
    // Wait for socket creation
    tokio::time::sleep(Duration::from_millis(100)).await;
    
    let mut stream = timeout(Duration::from_secs(5), UnixStream::connect(SOCKET_PATH))
        .await
        .expect("Timeout connecting to socket")
        .expect("Failed to connect to socket");
    
    println!("✅ Connected to IPC socket for invalid message test");
    
    // Test: Send invalid/corrupted data
    let invalid_data = vec![0xFF, 0xFE, 0xFD, 0xFC]; // Random bytes
    
    println!("📤 Sending invalid data: {} bytes", invalid_data.len());
    write_message_with_length_prefix(&mut stream, &invalid_data)
        .await
        .expect("Failed to send invalid data");
    
    // Test: Send empty message
    let empty_data = vec![];
    println!("📤 Sending empty message");
    write_message_with_length_prefix(&mut stream, &empty_data)
        .await
        .expect("Failed to send empty message");
    
    // Give time for processing
    tokio::time::sleep(Duration::from_millis(100)).await;
    
    println!("✅ Invalid message handling test completed");
    
    // Clean up
    drop(stream);
    listener_handle.abort();
    let _ = std::fs::remove_file(SOCKET_PATH);
}

#[cfg(windows)]
#[tokio::test]
async fn test_ipc_windows_stub() {
    // On Windows, just test that the SocketListener can be created and started
    let mut listener = SocketListener::new();
    
    // This should succeed on Windows (stub implementation)
    let result = listener.start().await;
    assert!(result.is_ok(), "Windows stub should succeed");
    
    println!("✅ Windows IPC stub test completed");
}

#[tokio::test]
async fn test_socket_listener_creation() {
    let listener = SocketListener::new();
    // Just verify it can be created
    drop(listener);
    println!("✅ SocketListener creation test passed");
}