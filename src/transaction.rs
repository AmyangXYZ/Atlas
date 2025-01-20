#[derive(Debug, Clone)]
pub struct Transaction {
    pub client_id: u16,
    pub data_name: String,
    pub operation: u8,
    pub signature: Vec<u8>,
    pub data: Vec<u8>,
}

impl Transaction {
    pub fn new(
        client_id: u16,
        data_name: String,
        operation: u8,
        signature: Vec<u8>,
        data: Vec<u8>,
    ) -> Self {
        Self {
            client_id,
            data_name,
            operation,
            signature,
            data,
        }
    }

    pub fn from_bytes(bytes: &[u8]) -> Self {
        Self {
            client_id: u16::from_le_bytes(bytes[0..2].try_into().unwrap()),
            data_name: String::from_utf8(bytes[2..].to_vec()).unwrap(),
            operation: bytes[2 + bytes[2..].len()],
            signature: bytes[3 + bytes[2..].len()..].to_vec(),
            data: bytes[3 + bytes[2..].len() + bytes[3 + bytes[2..].len()..].len()..].to_vec(),
        }
    }

    pub fn as_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        bytes.extend_from_slice(&self.client_id.to_le_bytes());
        bytes.extend_from_slice(&self.data_name.as_bytes());
        bytes.extend_from_slice(&self.operation.to_le_bytes());
        bytes.extend_from_slice(&self.signature);
        bytes.extend_from_slice(&self.data);
        bytes
    }
}
