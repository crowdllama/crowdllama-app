use std::process::Command;
use std::sync::{Arc, Mutex};

pub struct SidecarState {
    pub process_id: Option<u32>,
}

impl SidecarState {
    pub fn new() -> Self {
        Self { process_id: None }
    }

    pub fn spawn_sidecar(&mut self, binary_path: &str) -> Option<u32> {
        let child = Command::new(binary_path)
            .spawn()
            .expect("Failed to spawn crowdllama process");
        self.process_id = Some(child.id());
        println!("Spawned crowdllama sidecar process with ID: {}", child.id());
        Some(child.id())
    }

    pub fn kill_sidecar(&self) {
        if let Some(pid) = self.process_id {
            let _ = Command::new("kill").arg(pid.to_string()).output();
            println!("Killed crowdllama sidecar process with ID: {}", pid);
        }
    }
}

pub fn new_sidecar_state() -> Arc<Mutex<SidecarState>> {
    Arc::new(Mutex::new(SidecarState::new()))
} 