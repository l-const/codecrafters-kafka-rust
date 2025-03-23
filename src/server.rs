struct KafkServer;

impl KafkServer {
    pub fn new() -> KafkServer {
        KafkServer
    }

    pub fn start(&self) {
        println!("Starting Kafka server...");
    }
}
