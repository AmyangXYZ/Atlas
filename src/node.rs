pub struct AtlasNode {
    pub id: String,
    pub address: String,
}

impl AtlasNode {
    pub fn new(id: String, address: String) -> Self {
        Self { id, address }
    }
}
