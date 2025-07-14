import { useState, useEffect } from "react";
import reactLogo from "./assets/react.svg";
import { invoke } from "@tauri-apps/api/core";
import "./App.css";

interface JsonGenerateRequest {
  model: string;
  prompt: string;
  stream: boolean;
}

interface JsonGenerateResponse {
  model: string;
  response: string;
  done: boolean;
  worker_id: string;
}

type JsonBaseMessage = 
  | { type: "GenerateRequest"; data: JsonGenerateRequest }
  | { type: "GenerateResponse"; data: JsonGenerateResponse };

function App() {
  const [greetMsg, setGreetMsg] = useState("");
  const [name, setName] = useState("");
  const [ipcMessages, setIpcMessages] = useState<string[]>([]);
  const [prompt, setPrompt] = useState("Hello from React!");
  const [model, setModel] = useState("test-model");

  async function greet() {
    // Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
    setGreetMsg(await invoke("greet", { name }));
  }

  async function sendGenerateRequest() {
    const message: JsonBaseMessage = {
      type: "GenerateRequest",
      data: {
        model: model,
        prompt: prompt,
        stream: false,
      }
    };

    console.log("📤 Sending GenerateRequest to Rust:", message);
    addIpcMessage(`📤 Sending GenerateRequest: ${prompt}`);

    try {
      const result = await invoke<string>("send_ipc_message", { message });
      console.log("✅ Result from Rust:", result);
      addIpcMessage(`✅ ${result}`);
    } catch (error) {
      console.error("❌ Error sending message:", error);
      addIpcMessage(`❌ Error: ${error}`);
    }
  }

  async function sendGenerateResponse() {
    const message: JsonBaseMessage = {
      type: "GenerateResponse",
      data: {
        model: model,
        response: "This is a response from React!",
        done: true,
        worker_id: "react-worker-123",
      }
    };

    console.log("📤 Sending GenerateResponse to Rust:", message);
    addIpcMessage(`📤 Sending GenerateResponse from React`);

    try {
      const result = await invoke<string>("send_ipc_message", { message });
      console.log("✅ Result from Rust:", result);
      addIpcMessage(`✅ ${result}`);
    } catch (error) {
      console.error("❌ Error sending message:", error);
      addIpcMessage(`❌ Error: ${error}`);
    }
  }

  async function simulateReceiveMessage() {
    console.log("📨 Simulating receive message from IPC");
    addIpcMessage("📨 Requesting simulated IPC message...");

    try {
      const message = await invoke<JsonBaseMessage>("simulate_ipc_message");
      console.log("📨 Received message from Rust:", message);
      addIpcMessage(`📨 Received: ${JSON.stringify(message, null, 2)}`);
    } catch (error) {
      console.error("❌ Error receiving message:", error);
      addIpcMessage(`❌ Error: ${error}`);
    }
  }

  function addIpcMessage(msg: string) {
    setIpcMessages(prev => [...prev.slice(-9), `${new Date().toLocaleTimeString()}: ${msg}`]);
  }

  function clearMessages() {
    setIpcMessages([]);
    console.clear();
  }

  useEffect(() => {
    console.log("🚀 CrowdLLaMA IPC Testing Interface Ready");
    addIpcMessage("🚀 IPC Testing Interface Ready");
  }, []);

  return (
    <main className="container">
      <h1>CrowdLLaMA IPC Testing</h1>

      <div className="row">
        <a href="https://vitejs.dev" target="_blank">
          <img src="/vite.svg" className="logo vite" alt="Vite logo" />
        </a>
        <a href="https://tauri.app" target="_blank">
          <img src="/tauri.svg" className="logo tauri" alt="Tauri logo" />
        </a>
        <a href="https://reactjs.org" target="_blank">
          <img src={reactLogo} className="logo react" alt="React logo" />
        </a>
      </div>

      {/* Original Greet Section */}
      <section style={{ marginBottom: "2rem", padding: "1rem", border: "1px solid #333", borderRadius: "8px" }}>
        <h2>Original Greet Test</h2>
        <form
          className="row"
          onSubmit={(e) => {
            e.preventDefault();
            greet();
          }}
        >
          <input
            id="greet-input"
            onChange={(e) => setName(e.currentTarget.value)}
            placeholder="Enter a name..."
          />
          <button type="submit">Saludar</button>
        </form>
        <p>{greetMsg}</p>
      </section>

      {/* IPC Testing Section */}
      <section style={{ marginBottom: "2rem", padding: "1rem", border: "1px solid #333", borderRadius: "8px" }}>
        <h2>IPC Message Testing</h2>
        
        <div style={{ marginBottom: "1rem" }}>
          <label>
            Model: 
            <input 
              value={model} 
              onChange={(e) => setModel(e.target.value)}
              placeholder="Model name"
              style={{ marginLeft: "0.5rem", marginRight: "1rem" }}
            />
          </label>
          <label>
            Prompt: 
            <input 
              value={prompt} 
              onChange={(e) => setPrompt(e.target.value)}
              placeholder="Enter prompt..."
              style={{ marginLeft: "0.5rem", width: "300px" }}
            />
          </label>
        </div>

        <div style={{ display: "flex", gap: "1rem", marginBottom: "1rem" }}>
          <button onClick={sendGenerateRequest} style={{ backgroundColor: "#4CAF50" }}>
            📤 Send GenerateRequest
          </button>
          <button onClick={sendGenerateResponse} style={{ backgroundColor: "#2196F3" }}>
            📤 Send GenerateResponse  
          </button>
          <button onClick={simulateReceiveMessage} style={{ backgroundColor: "#FF9800" }}>
            📨 Simulate Receive
          </button>
          <button onClick={clearMessages} style={{ backgroundColor: "#f44336" }}>
            🗑️ Clear Messages
          </button>
        </div>

        <div style={{ 
          backgroundColor: "#1a1a1a", 
          padding: "1rem", 
          borderRadius: "4px", 
          fontFamily: "monospace",
          fontSize: "12px",
          maxHeight: "300px",
          overflowY: "auto"
        }}>
          <h3>IPC Message Log:</h3>
          {ipcMessages.length === 0 ? (
            <p style={{ color: "#666" }}>No messages yet...</p>
          ) : (
            ipcMessages.map((msg, idx) => (
              <div key={idx} style={{ marginBottom: "0.25rem", color: "#fff" }}>
                {msg}
              </div>
            ))
          )}
        </div>
      </section>

      <section style={{ padding: "1rem", border: "1px solid #333", borderRadius: "8px", fontSize: "12px" }}>
        <h3>Testing Instructions:</h3>
        <ol>
          <li><strong>Send GenerateRequest:</strong> Sends a protobuf GenerateRequest to the IPC socket</li>
          <li><strong>Send GenerateResponse:</strong> Sends a protobuf GenerateResponse to the IPC socket</li>
          <li><strong>Simulate Receive:</strong> Simulates receiving a message from IPC and logs it</li>
          <li><strong>Check Console:</strong> Open browser DevTools to see detailed logging</li>
          <li><strong>Check Rust Logs:</strong> Check the terminal running the Tauri app for Rust-side logs</li>
        </ol>
        <p><strong>Note:</strong> IPC socket communication only works on Unix systems (Linux/macOS). Windows will show stub messages.</p>
      </section>
    </main>
  );
}

export default App;
