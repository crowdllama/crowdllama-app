# CrowdLLaMA IPC Testing Guide

This guide provides comprehensive testing for the IPC (Inter-Process Communication) functionality between React frontend, Rust backend, and Unix socket communication.

## 🏗️ Architecture Overview

```
React Frontend ↔ Tauri Commands ↔ Rust Backend ↔ Unix Socket ↔ External Processes
```

- **React → Rust**: JSON messages via Tauri commands
- **Rust → IPC**: Protobuf messages via Unix domain socket
- **IPC → Rust**: Protobuf messages received and parsed

## 🧪 Testing Components

### 1. Rust Integration Tests (`src-tauri/tests/ipc_integration_test.rs`)
- **Purpose**: Test Unix socket communication with protobuf messages
- **Tests**:
  - `test_ipc_generate_request_response_flow()`: Full GenerateRequest/Response flow
  - `test_ipc_invalid_message_handling()`: Error handling for invalid messages
  - `test_ipc_windows_stub()`: Windows compatibility test
  - `test_socket_listener_creation()`: Basic SocketListener creation

### 2. React Frontend Interface (`src/App.tsx`)
- **Purpose**: User interface for testing React ↔ Rust communication
- **Features**:
  - Send GenerateRequest to IPC socket
  - Send GenerateResponse to IPC socket
  - Simulate receiving messages from IPC
  - Real-time message logging
  - Console logging for debugging

### 3. Python Test Script (`test_ipc.py`)
- **Purpose**: External testing of Unix socket communication
- **Features**:
  - Direct socket connection testing
  - Message length prefix handling
  - Batch message testing
  - Error handling and diagnostics

## 🚀 How to Run Tests

### Prerequisites
```bash
# Ensure system dependencies are installed (Linux only)
sudo apt-get install -y libwebkit2gtk-4.1-dev build-essential curl wget libssl-dev libgtk-3-dev

# Install Rust and Node.js dependencies
cd src-tauri
cargo build
cd ..
npm install  # or bun install
```

### 1. Run Rust Integration Tests
```bash
cd src-tauri
export PATH="/usr/local/cargo/bin:$PATH"
cargo test ipc_integration_test --lib -- --nocapture
```

**Expected Output:**
- ✅ Socket creation and connection
- ✅ Message sending with length prefixes
- ✅ Protobuf encoding/decoding
- ✅ Multiple message handling

### 2. Start the Tauri Application
```bash
# Terminal 1: Start the development server
npm run tauri dev
# or
bun run tauri dev
```

**Look for these logs:**
```
🚀 CrowdLLaMA IPC Testing Interface Ready
Socket listener started on /tmp/crowdllama.sock
```

### 3. Test React Frontend Interface
1. **Open the Tauri application** (should open automatically)
2. **Navigate to the IPC Testing section**
3. **Test the buttons**:
   - `📤 Send GenerateRequest`: Tests React → Rust → IPC flow
   - `📤 Send GenerateResponse`: Tests response message flow
   - `📨 Simulate Receive`: Tests IPC → Rust → React flow
   - `🗑️ Clear Messages`: Clears the log

**Expected Behavior:**
- Messages appear in the UI log
- Console logs show detailed message flow
- Rust terminal shows received protobuf messages

### 4. Test with Python Script
```bash
# Terminal 2: Run the external test script
python3 test_ipc.py
```

**Expected Output:**
```
🧪 CrowdLLaMA IPC Socket Test Script
🔌 Attempting to connect to IPC socket...
✅ Connected to /tmp/crowdllama.sock
📤 Test 1: Sending GenerateRequest
📤 Sent message: 123 bytes
📤 Test 2: Sending GenerateResponse
📤 Sent message: 145 bytes
📤 Test 3: Sending batch messages
✅ All test messages sent successfully!
```

### 5. Test with cURL (Advanced)
```bash
# Test socket connectivity (this will fail but shows socket exists)
echo "test" | nc -U /tmp/crowdllama.sock

# Check socket file exists
ls -la /tmp/crowdllama.sock
```

## 📊 Expected Log Outputs

### Rust Backend Logs
```
Socket listener started on /tmp/crowdllama.sock
New connection accepted
📏 Reading message of length: 45 bytes
✅ Received protobuf message: 🔄 GenerateRequest: model=test-model, prompt=Hello from React!, stream=false
🔄 Received message from React: GenerateRequest { data: JsonGenerateRequest { model: "test-model", prompt: "Hello from React!", stream: false } }
📦 Encoded protobuf message: 45 bytes
📤 Sent 45 bytes to IPC socket
✅ Successfully sent message to IPC socket
```

### React Frontend Logs (Browser Console)
```
🚀 CrowdLLaMA IPC Testing Interface Ready
📤 Sending GenerateRequest to Rust: {type: "GenerateRequest", data: {model: "test-model", prompt: "Hello from React!", stream: false}}
✅ Result from Rust: Message sent successfully
📨 Simulating receive message from IPC
📨 Received message from Rust: {type: "GenerateResponse", data: {model: "simulated-model", response: "This is a simulated response from the IPC system", done: true, worker_id: "worker-sim-123"}}
```

### Python Script Logs
```
🔌 Attempting to connect to IPC socket...
✅ Connected to /tmp/crowdllama.sock
📤 Test 1: Sending GenerateRequest
📤 Sent message: 123 bytes
📤 Test 2: Sending GenerateResponse  
📤 Sent message: 145 bytes
🎉 Test completed successfully!
```

## 🐛 Troubleshooting

### Common Issues

#### 1. Socket Not Found
```
❌ Socket file not found: /tmp/crowdllama.sock
```
**Solution**: Ensure the Tauri application is running and IPC listener started.

#### 2. Connection Refused
```
❌ Connection refused to: /tmp/crowdllama.sock
```
**Solution**: Check that the socket listener is running and not crashed.

#### 3. Permission Denied
```
❌ Permission denied: /tmp/crowdllama.sock
```
**Solution**: Check file permissions or run with appropriate user.

#### 4. Windows Compatibility
```
⚠️ Unix domain sockets are not supported on Windows
```
**Expected**: Windows uses stub implementation for testing.

### Debugging Steps

1. **Check socket file exists**:
   ```bash
   ls -la /tmp/crowdllama.sock
   ```

2. **Monitor socket connections**:
   ```bash
   sudo netstat -ax | grep crowdllama
   ```

3. **Check Tauri logs**:
   - Look for "Socket listener started" message
   - Check for any error messages in the terminal

4. **Verify protobuf encoding**:
   - Check byte lengths in logs
   - Verify message structure in Rust output

## 🔍 Message Flow Verification

### Complete Flow Test
1. **Start Tauri app** → Socket created
2. **Click "Send GenerateRequest"** → React → Rust → Socket
3. **Run Python script** → External → Socket → Rust logs
4. **Click "Simulate Receive"** → Rust → React → UI update

### Success Indicators
- ✅ Socket file created at `/tmp/crowdllama.sock`
- ✅ Messages appear in all three logs (React, Rust, Python)
- ✅ Protobuf encoding/decoding works correctly
- ✅ Length prefixes handled properly
- ✅ No connection errors or timeouts

## 📝 Test Results Template

```
## Test Run Results

**Date**: [DATE]
**Platform**: [Linux/macOS/Windows]
**Rust Version**: [VERSION]
**Node Version**: [VERSION]

### Test Results
- [ ] Rust integration tests pass
- [ ] Tauri app starts successfully
- [ ] Socket file created
- [ ] React buttons work
- [ ] Python script connects
- [ ] Messages logged correctly
- [ ] No errors in console

### Issues Found
[List any issues or errors encountered]

### Notes
[Additional observations or comments]
```

## 🎯 Next Steps

After successful testing:
1. **Implement real protobuf communication** (replace JSON test messages)
2. **Add bidirectional message handling** (IPC → React notifications)
3. **Implement message queuing** for high-volume scenarios
4. **Add authentication/security** for production use
5. **Create performance benchmarks** for message throughput