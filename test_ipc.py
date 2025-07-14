#!/usr/bin/env python3
"""
Test script for CrowdLLaMA IPC socket communication.
This script connects to the Unix socket and sends protobuf messages.
"""

import socket
import struct
import time
import sys

# Simple protobuf-like message encoding for testing
def create_test_message(message_type: str, data: dict) -> bytes:
    """Create a simple test message (not real protobuf, just for testing)"""
    import json
    
    # Create a simple JSON message for testing
    test_msg = {
        "type": message_type,
        "data": data,
        "timestamp": time.time()
    }
    
    return json.dumps(test_msg).encode('utf-8')

def send_message_with_length_prefix(sock: socket.socket, message: bytes):
    """Send a message with length prefix (4 bytes, big-endian)"""
    length = struct.pack('>I', len(message))  # Big-endian 4-byte length
    sock.send(length)
    sock.send(message)
    print(f"📤 Sent message: {len(message)} bytes")

def test_ipc_socket():
    """Test the IPC socket communication"""
    socket_path = "/tmp/crowdllama.sock"
    
    print("🔌 Attempting to connect to IPC socket...")
    
    try:
        # Create Unix domain socket
        sock = socket.socket(socket.AF_UNIX, socket.SOCK_STREAM)
        
        # Connect to the socket
        sock.connect(socket_path)
        print(f"✅ Connected to {socket_path}")
        
        # Test 1: Send GenerateRequest
        print("\n📤 Test 1: Sending GenerateRequest")
        request_data = {
            "model": "test-model-python",
            "prompt": "Hello from Python test script!",
            "stream": False
        }
        request_msg = create_test_message("GenerateRequest", request_data)
        send_message_with_length_prefix(sock, request_msg)
        
        time.sleep(0.1)  # Small delay
        
        # Test 2: Send GenerateResponse
        print("\n📤 Test 2: Sending GenerateResponse")
        response_data = {
            "model": "test-model-python",
            "response": "This is a response from Python!",
            "done": True,
            "worker_id": "python-worker-123"
        }
        response_msg = create_test_message("GenerateResponse", response_data)
        send_message_with_length_prefix(sock, response_msg)
        
        time.sleep(0.1)  # Small delay
        
        # Test 3: Send multiple messages
        print("\n📤 Test 3: Sending batch messages")
        for i in range(3):
            batch_data = {
                "model": f"batch-model-{i}",
                "prompt": f"Batch message {i} from Python",
                "stream": i % 2 == 0
            }
            batch_msg = create_test_message("GenerateRequest", batch_data)
            send_message_with_length_prefix(sock, batch_msg)
            time.sleep(0.05)  # Small delay between messages
        
        print("\n✅ All test messages sent successfully!")
        print("📋 Check the Rust application logs to see if messages were received and parsed.")
        
    except FileNotFoundError:
        print(f"❌ Socket file not found: {socket_path}")
        print("💡 Make sure the Tauri application is running and the IPC socket is created.")
        return False
        
    except ConnectionRefusedError:
        print(f"❌ Connection refused to: {socket_path}")
        print("💡 Make sure the IPC socket listener is running.")
        return False
        
    except Exception as e:
        print(f"❌ Error: {e}")
        return False
        
    finally:
        sock.close()
        print("🔌 Socket connection closed")
    
    return True

def main():
    print("🧪 CrowdLLaMA IPC Socket Test Script")
    print("=" * 50)
    
    if len(sys.argv) > 1 and sys.argv[1] == "--help":
        print("""
Usage: python3 test_ipc.py

This script tests the IPC socket communication by:
1. Connecting to /tmp/crowdllama.sock
2. Sending test messages with length prefixes
3. Testing GenerateRequest and GenerateResponse flows

Make sure to:
1. Start the Tauri application first
2. Ensure the IPC socket listener is running
3. Check the Rust application logs for received messages
        """)
        return
    
    print("🚀 Starting IPC socket test...")
    success = test_ipc_socket()
    
    if success:
        print("\n🎉 Test completed successfully!")
        print("📝 Next steps:")
        print("   1. Check the Tauri application terminal for Rust-side logs")
        print("   2. Verify that messages were received and parsed correctly")
        print("   3. Test the React frontend buttons for full round-trip testing")
    else:
        print("\n💥 Test failed!")
        print("🔧 Troubleshooting:")
        print("   1. Ensure the Tauri application is running")
        print("   2. Check that the IPC socket listener started successfully")
        print("   3. Verify the socket path: /tmp/crowdllama.sock")

if __name__ == "__main__":
    main()