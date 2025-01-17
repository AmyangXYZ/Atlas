use crate::protocol::{
    AckPayload, JoinRequestPayload, JoinResponsePayload, KeyPayload, Packet, PacketType,
    ACK_TIMEOUT, MAX_RETRIES, ORCHESTRATOR_ID, PACKET_BUFFER_SIZE,
};
use crate::{cache::InMemoryCache, port::Port, port::UdpPort};
use ring::rand;
use ring::signature::{self, Ed25519KeyPair, KeyPair, Signature, UnparsedPublicKey};
use std::collections::HashMap;
use std::time::{Duration, Instant};

pub struct Node {
    pub id: u16,
    pub cache: InMemoryCache,
    port: UdpPort,
    orchestrator_address: String,
    is_orchestrator: bool,
    joined: bool,
    addr_table: HashMap<u16, String>,
    local_key_pair: Ed25519KeyPair,
    peer_keys: HashMap<u16, Vec<u8>>,
    pending_acks: HashMap<u32, (u8, Instant, Packet)>,
}

impl Node {
    pub fn new(id: u16, address: &str, orchestrator_address: &str) -> Self {
        let rng = rand::SystemRandom::new();
        let pkcs8_bytes = Ed25519KeyPair::generate_pkcs8(&rng).unwrap();

        Self {
            id,
            cache: InMemoryCache::new(),
            port: UdpPort::bind(&address, Duration::from_millis(10)).unwrap(),
            orchestrator_address: orchestrator_address.to_string(),
            is_orchestrator: id == ORCHESTRATOR_ID,
            joined: id == ORCHESTRATOR_ID,
            addr_table: HashMap::new(),
            local_key_pair: Ed25519KeyPair::from_pkcs8(pkcs8_bytes.as_ref()).unwrap(),
            peer_keys: HashMap::new(),
            pending_acks: HashMap::new(),
        }
    }

    pub fn run(&mut self) {
        if !self.is_orchestrator {
            self.send_join_request(&self.orchestrator_address.clone());
        }

        let mut buffer = [0; PACKET_BUFFER_SIZE];
        loop {
            self.check_ack_timeouts();

            let Some((size, addr)) = self.port.receive(&mut buffer) else {
                continue;
            };
            let Some(packet) = Packet::from_bytes(&buffer[..size]) else {
                continue;
            };
            self.addr_table.insert(packet.src, addr.to_string());

            println!(
                "[{:?}] Received {:?}-0x{:X} from {:?}",
                self.id,
                PacketType::from(packet.packet_type),
                packet.packet_id,
                packet.src
            );
            if PacketType::from(packet.packet_type) != PacketType::Ack {
                self.reply_ack(&packet);
            }
            match PacketType::from(packet.packet_type) {
                PacketType::Ack => self.handle_ack(&packet),
                PacketType::JoinRequest => self.handle_join_request(packet),
                PacketType::JoinResponse => self.handle_join_response(&packet),
                PacketType::KeyRequest => self.handle_key_request(packet),
                PacketType::KeyResponse => self.handle_key_response(&packet),
                _ => continue,
            }
        }
    }

    pub fn sign(&self, message: String) -> Signature {
        self.local_key_pair.sign(message.as_bytes())
    }

    pub fn verify(&self, message: &str, sig_bytes: &[u8], public_key: &[u8]) -> bool {
        let verify_key = UnparsedPublicKey::new(&signature::ED25519, public_key);
        verify_key.verify(message.as_bytes(), sig_bytes).is_ok()
    }

    fn reply_ack(&mut self, packet: &Packet) {
        let ack_packet = Packet::new(
            self.id,
            packet.src,
            PacketType::Ack,
            AckPayload::new(packet.packet_id).as_bytes(),
        );
        self.send(&ack_packet);
    }

    fn send_join_request(&mut self, orchestrator_address: &str) {
        self.addr_table.insert(0, orchestrator_address.to_string());
        let packet = Packet::new(
            self.id,
            0,
            PacketType::JoinRequest,
            JoinRequestPayload::new(self.id, self.local_key_pair.public_key().as_ref().to_vec())
                .as_bytes(),
        );
        self.send(&packet);
    }

    fn handle_ack(&mut self, packet: &Packet) {
        let ack_payload = AckPayload::from_bytes(&packet.payload);
        self.pending_acks.remove(&ack_payload.packet_id);
    }

    fn handle_join_request(&mut self, packet: Packet) {
        let join_request_payload = JoinRequestPayload::from_bytes(&packet.payload);
        self.peer_keys
            .insert(join_request_payload.node_id, join_request_payload.key);

        let packet = Packet::new(
            self.id,
            join_request_payload.node_id,
            PacketType::JoinResponse,
            JoinResponsePayload::new(true).as_bytes(),
        );

        self.send(&packet);
    }

    fn handle_join_response(&mut self, packet: &Packet) {
        let join_response_payload = JoinResponsePayload::from_bytes(&packet.payload);
        self.joined = join_response_payload.permission;
    }

    fn handle_key_request(&mut self, packet: Packet) {
        let key_payload = KeyPayload::from_bytes(&packet.payload);
        if let Some(key) = self.peer_keys.get(&key_payload.node_id) {
            let packet = Packet::new(
                self.id,
                key_payload.node_id,
                PacketType::KeyResponse,
                key.to_vec(),
            );
            self.send(&packet);
        }
    }

    fn handle_key_response(&mut self, packet: &Packet) {
        let key_payload = KeyPayload::from_bytes(&packet.payload);
        self.peer_keys.insert(key_payload.node_id, key_payload.key);
    }

    fn send(&mut self, packet: &Packet) {
        println!(
            "[{:?}] Sending {:?}-0x{:X} to {:?}",
            self.id,
            PacketType::from(packet.packet_type),
            packet.packet_id,
            packet.dst
        );
        if let Some(dst_addr) = self.addr_table.get(&packet.dst) {
            self.port.send(dst_addr, &packet.as_bytes());
            if PacketType::from(packet.packet_type) != PacketType::Ack {
                self.pending_acks
                    .insert(packet.packet_id, (0, Instant::now(), packet.clone()));
            }
        }
    }

    fn check_ack_timeouts(&mut self) {
        let now = Instant::now();
        let mut to_retry: Vec<_> = self
            .pending_acks
            .iter()
            .filter(|(_, (_, sent_time, _))| {
                now.duration_since(*sent_time) >= Duration::from_millis(ACK_TIMEOUT)
            })
            .map(|(&id, _)| id)
            .collect();

        for packet_id in to_retry.drain(..) {
            let (retries, _, packet) = self.pending_acks.remove(&packet_id).unwrap();
            if retries < MAX_RETRIES {
                if let Some(dst_addr) = self.addr_table.get(&packet.dst) {
                    println!(
                        "[{:?}] Retransmitting {:?}-0x{:X} (attempt {})",
                        self.id,
                        PacketType::from(packet.packet_type),
                        packet_id,
                        retries + 2
                    );
                    self.port.send(dst_addr, &packet.as_bytes());
                    self.pending_acks
                        .insert(packet_id, (retries + 1, Instant::now(), packet));
                }
            } else {
                println!(
                    "[{:?}] Packet {:?}-0x{:X} failed after 3 retries",
                    self.id,
                    PacketType::from(packet.packet_type),
                    packet_id
                );
            }
        }
    }
}
