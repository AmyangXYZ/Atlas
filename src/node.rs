use crate::{cache::InMemoryCache, port::Port, port::UdpPort};
use ring::rand;
use ring::signature::{self, Ed25519KeyPair, KeyPair, Signature, UnparsedPublicKey};
use std::collections::HashMap;

pub struct Node {
    pub id: u16,
    pub cache: InMemoryCache,
    pub port: UdpPort,
    pub local_key_pair: Ed25519KeyPair,
    pub peer_keys: HashMap<String, Vec<u8>>,
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

    pub fn serve(&self) {
        let mut buffer = [0; 1024];
        while let Some((size, _)) = self.port.receive(&mut buffer) {
            let message = String::from_utf8_lossy(&buffer[..size]);
            println!("Received message: {}", message);
        }
    }

    pub fn public_key(&self) -> Vec<u8> {
        self.local_key_pair.public_key().as_ref().to_vec()
    }

    pub fn sign(&self, message: String) -> Signature {
        self.local_key_pair.sign(message.as_bytes())
    }

    pub fn verify(&self, message: String, sig: Signature) -> bool {
        let verify_key = UnparsedPublicKey::new(&signature::ED25519, self.public_key());
        verify_key.verify(message.as_bytes(), sig.as_ref()).is_ok()
    }
}
