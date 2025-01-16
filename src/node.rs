use crate::protocol::{
    JoinRequestPayload, JoinResponsePayload, Packet, PacketType, PACKET_BUFFER_SIZE,
};
use crate::{cache::InMemoryCache, port::Port, port::UdpPort};
use ring::rand;
use ring::signature::{self, Ed25519KeyPair, KeyPair, Signature, UnparsedPublicKey};
use std::collections::HashMap;

pub struct Node {
    pub id: u16,
    pub cache: InMemoryCache,
    port: UdpPort,
    local_key_pair: Ed25519KeyPair,
    peer_keys: HashMap<u16, Vec<u8>>,
}

impl Node {
    pub fn new(id: u16, address: &str) -> Self {
        let rng = rand::SystemRandom::new();
        let pkcs8_bytes = Ed25519KeyPair::generate_pkcs8(&rng).unwrap();

        Self {
            id,
            cache: InMemoryCache::new(),
            port: UdpPort::bind(&address).unwrap(),
            local_key_pair: Ed25519KeyPair::from_pkcs8(pkcs8_bytes.as_ref()).unwrap(),
            peer_keys: HashMap::new(),
        }
    }

    pub fn send(&self, dst: &str, message: &[u8]) {
        self.port.send(dst, message);
    }

    pub fn run(&mut self) {
        let mut buffer = [0; PACKET_BUFFER_SIZE];
        while let Some((size, _)) = self.port.receive(&mut buffer) {
            if let Some(packet) = Packet::from_bytes(&buffer[..size]) {
                match PacketType::from(packet.packet_type) {
                    PacketType::KeyResponse => {
                        let sig_bytes = &buffer[..size];
                        println!("Received signature: {:?}", sig_bytes);
                        if let Some(public_key) = self.peer_keys.get(&2) {
                            let verified = self.verify("hello world", sig_bytes, public_key);
                            println!("Verified: {}", verified);
                        } else {
                            println!("Peer key not found");
                        }
                    }
                    _ => continue,
                }
            }
        }
    }

    pub fn join(&mut self, orchestrator_address: &str) {
        let packet = Packet::new(
            self.id,
            0,
            PacketType::JoinRequest,
            JoinRequestPayload::new(self.id, self.public_key()).as_bytes(),
        );
        self.port.send(orchestrator_address, &packet.as_bytes());

        let mut buffer = [0; PACKET_BUFFER_SIZE];
        if let Some((size, _)) = self.port.receive(&mut buffer) {
            if let Some(packet) = Packet::from_bytes(&buffer[..size]) {
                if PacketType::from(packet.packet_type) == PacketType::JoinResponse {
                    let join_response_payload = JoinResponsePayload::from_bytes(&packet.payload);
                    if join_response_payload.permission {
                        println!("Joined successfully");
                    } else {
                        println!("Failed to join");
                    }
                }
            }
        }
    }

    pub fn public_key(&self) -> Vec<u8> {
        self.local_key_pair.public_key().as_ref().to_vec()
    }

    pub fn sign(&self, message: String) -> Signature {
        self.local_key_pair.sign(message.as_bytes())
    }

    pub fn verify(&self, message: &str, sig_bytes: &[u8], public_key: &[u8]) -> bool {
        let verify_key = UnparsedPublicKey::new(&signature::ED25519, public_key);
        verify_key.verify(message.as_bytes(), sig_bytes).is_ok()
    }
}
