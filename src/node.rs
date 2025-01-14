pub struct AtlasNode {
    pub id: String,
    pub address: String,
}

impl AtlasNode {
    pub fn new(id: String, address: String) -> Self {
        Self { id, address }
    }
    pub fn sign(self, message: String) -> String {
        let signature = self.private_key.sign(message.as_bytes());
        signature.to_string()
    }
}
