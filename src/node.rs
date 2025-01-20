use crate::protocol::{
    AckPayload, Packet, PacketType, ProbePayload, SyncPayload, ACK_TIMEOUT, MAX_RETRIES,
    PACKET_BUFFER_SIZE,
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
    is_leader: bool,
    addr_table: HashMap<u16, String>,
    key_pair: Ed25519KeyPair,
    peer_keys: HashMap<u16, Vec<u8>>,
    pending_acks: HashMap<u32, (u8, Instant, Packet)>,
}

impl Node {
    pub fn new(id: u16, address: &str) -> Self {
        let rng = rand::SystemRandom::new();
        let pkcs8_bytes = Ed25519KeyPair::generate_pkcs8(&rng).unwrap();

        Self {
            id,
            cache: InMemoryCache::new(),
            port: UdpPort::bind(&address, Duration::from_millis(10)).unwrap(),
            is_leader: id == 0,
            addr_table: HashMap::new(),
            key_pair: Ed25519KeyPair::from_pkcs8(pkcs8_bytes.as_ref()).unwrap(),
            peer_keys: HashMap::new(),
            pending_acks: HashMap::new(),
        }
    }

    pub fn run(&mut self) {
        if !self.is_leader {
            self.send_probe("127.0.0.1:8080");
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
                PacketType::Probe => self.handle_probe(packet),
                PacketType::Sync => self.handle_sync(&packet),
                _ => continue,
            }
        }
    }

    pub fn sign(&self, message: String) -> Signature {
        self.key_pair.sign(message.as_bytes())
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

    fn send_probe(&mut self, orchestrator_address: &str) {
        self.addr_table.insert(0, orchestrator_address.to_string());
        let packet = Packet::new(
            self.id,
            0,
            PacketType::Probe,
            ProbePayload::new(self.id, self.key_pair.public_key().as_ref().to_vec()).as_bytes(),
        );
        self.send(&packet);
    }

    fn handle_ack(&mut self, packet: &Packet) {
        let ack_payload = AckPayload::from_bytes(&packet.payload);
        self.pending_acks.remove(&ack_payload.packet_id);
    }

    fn handle_probe(&mut self, packet: Packet) {
        let probe_payload = ProbePayload::from_bytes(&packet.payload);
        self.peer_keys
            .insert(probe_payload.node_id, probe_payload.public_key.clone());

        let packet = Packet::new(
            self.id,
            probe_payload.node_id,
            PacketType::Sync,
            SyncPayload::new(
                self.id,
                self.key_pair.public_key().as_ref().to_vec(),
                0,
                [0; 32],
                0,
            )
            .as_bytes(),
        );

        self.send(&packet);
    }

    fn handle_sync(&mut self, packet: &Packet) {
        let sync_payload = SyncPayload::from_bytes(&packet.payload);
        self.peer_keys
            .insert(sync_payload.node_id, sync_payload.public_key.clone());
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
