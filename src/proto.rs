pub mod hello_world {
    tonic::include_proto!("helloworld");
}

impl From<String> for hello_world::HelloReply {
    fn from(message: String) -> Self {
        Self { message }
    }
}
