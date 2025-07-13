#[cfg(test)]
mod tests {
    use super::*;
    use crowdllama_app_lib::ipc::SocketListener;

    #[test]
    fn test_socket_listener_new() {
        let listener = SocketListener::new();
        // Just check that the type is constructed
        let _ = listener;
    }
} 