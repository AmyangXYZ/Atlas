pub fn serialize_hash<S>(hash: &[u8; 32], serializer: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    let hex = hash
        .iter()
        .map(|b| format!("{:02x}", b))
        .collect::<String>();
    serializer.serialize_str(&hex)
}

pub fn hex_string(bytes: &[u8]) -> String {
    bytes
        .iter()
        .map(|b| format!("{:02x}", b))
        .collect::<String>()
}
