use crate::block::{Block, BLOCK_PERIOD};
use crate::cache::CacheOperation;
use crate::protocol::{
    AckPayload, BlockPayload, ChainPayload, DataPayload, Packet, PacketType, ProbePayload,
    SyncPayload, TransactionPayload, ACK_TIMEOUT, MAX_RETRIES, PACKET_BUFFER_SIZE,
};
use crate::transaction::Transaction;
use crate::{cache::Cache, cache::InMemoryCache};
use ring::rand;
use ring::signature::{self, Ed25519KeyPair, KeyPair, Signature, UnparsedPublicKey};
use std::collections::HashMap;
use std::net::UdpSocket;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

fn system_log(node_id: u16, message: impl std::fmt::Display) {
    if true {
        println!("<SYSTEM> {} {}", node_id, message);
    }
}

fn network_log(node_id: u16, message: impl std::fmt::Display) {
    if true {
        println!("<PACKET> {} {}", node_id, message);
    }
}

pub struct Node {
    id: u16,
    socket: UdpSocket,
    addr_table: HashMap<u16, String>,
    pending_acks: HashMap<u32, (u8, Instant, Packet)>,

    cache: InMemoryCache,
    key_pair: Ed25519KeyPair,
    peer_public_keys: HashMap<u16, Vec<u8>>,
    leader: u16,
    pending_transactions: HashMap<[u8; 32], Transaction>,
    chain: Vec<Block>,
}

impl Node {
    pub fn new(id: u16, address: &str) -> Self {
        let rng = rand::SystemRandom::new();
        let pkcs8_bytes = Ed25519KeyPair::generate_pkcs8(&rng).unwrap();
        let socket = UdpSocket::bind(address).expect("Failed to bind to address");
        socket
            .set_read_timeout(Some(Duration::from_millis(10)))
            .unwrap();
        Self {
            id,
            socket,
            addr_table: HashMap::new(),
            pending_acks: HashMap::new(),

            cache: InMemoryCache::new(),
            leader: 0,
            key_pair: Ed25519KeyPair::from_pkcs8(pkcs8_bytes.as_ref()).unwrap(),
            peer_public_keys: HashMap::new(),
            pending_transactions: HashMap::new(),
            chain: Vec::new(),
        }
    }

    pub fn run(&mut self) {
        if self.id != self.leader {
            self.send_probe("127.0.0.1:8080");
        }
        let mut buffer = [0; PACKET_BUFFER_SIZE];
        loop {
            self.check_ack_timeouts();

            if self.id == self.leader {
                let now = SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap()
                    .as_secs();

                if self.chain.is_empty() {
                    self.create_block();
                } else if now - self.chain.last().unwrap().timestamp >= BLOCK_PERIOD {
                    self.create_block();
                }
            }

            let Ok((size, addr)) = self.socket.recv_from(&mut buffer) else {
                continue;
            };
            let Some(packet) = Packet::from_bytes(&buffer[..size]) else {
                continue;
            };
            self.addr_table.insert(packet.src, addr.to_string());

            network_log(
                self.id,
                format!(
                    "Received {:?}-0x{:X} from {:?}",
                    PacketType::from(packet.packet_type),
                    packet.packet_id,
                    packet.src
                ),
            );
            if PacketType::from(packet.packet_type) != PacketType::Ack {
                self.reply_ack(&packet);
            }
            match PacketType::from(packet.packet_type) {
                PacketType::Ack => self.handle_ack(&packet),
                PacketType::Probe => self.handle_probe(packet),
                PacketType::Sync => self.handle_sync(&packet),
                PacketType::SetData => self.handle_set_data(&packet),
                PacketType::GetData => self.handle_get_data(&packet),
                PacketType::GetChain => self.handle_get_chain(&packet),
                PacketType::Chain => self.handle_chain(&packet),
                PacketType::Transaction => self.handle_transaction(&packet),
                PacketType::Block => self.handle_block(&packet),
            }
        }
    }

    fn sign(&self, message: Vec<u8>) -> Signature {
        self.key_pair.sign(message.as_ref())
    }

    fn verify(&self, message: &[u8], sig_bytes: &[u8], public_key: &[u8]) -> bool {
        let verify_key = UnparsedPublicKey::new(&signature::ED25519, public_key);
        verify_key.verify(message, sig_bytes).is_ok()
    }

    fn create_block(&mut self) {
        let transactions = self.pending_transactions.values().cloned().collect();
        self.pending_transactions.clear();
        let block = Block::new(
            transactions,
            if self.chain.is_empty() {
                [0; 32]
            } else {
                self.chain.last().unwrap().merkle_root
            },
        );

        let block_payload = BlockPayload::new(block.clone());
        system_log(
            self.id,
            format!(
                "Created block #{:?} {}",
                self.chain.len(),
                hex_string(&block.merkle_root)
            ),
        );
        self.chain.push(block);

        let peers: Vec<u16> = self.peer_public_keys.keys().copied().collect();
        for peer in peers {
            let block_packet =
                Packet::new(self.id, peer, PacketType::Block, block_payload.as_bytes());
            self.send(&block_packet);
        }
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
        let probe_packet = Packet::new(
            self.id,
            0,
            PacketType::Probe,
            ProbePayload::new(
                self.id,
                self.key_pair.public_key().as_ref().try_into().unwrap(),
            )
            .as_bytes(),
        );
        self.send(&probe_packet);
    }

    fn handle_ack(&mut self, packet: &Packet) {
        let ack_payload = AckPayload::from_bytes(&packet.payload);
        self.pending_acks.remove(&ack_payload.packet_id);
    }

    fn handle_probe(&mut self, packet: Packet) {
        let probe_payload = ProbePayload::from_bytes(&packet.payload);
        self.peer_public_keys
            .insert(probe_payload.node_id, probe_payload.public_key.to_vec());

        system_log(
            self.id,
            format!(
                "Added public key of {:?}: {}",
                probe_payload.node_id,
                hex_string(&probe_payload.public_key)
            ),
        );

        if !self.chain.is_empty() {
            let sync_packet = Packet::new(
                self.id,
                probe_payload.node_id,
                PacketType::Sync,
                SyncPayload::new(
                    self.id,
                    self.key_pair.public_key().as_ref().try_into().unwrap(),
                    self.chain.len() as u32,
                    self.chain.last().unwrap().merkle_root,
                    self.chain.last().unwrap().timestamp,
                )
                .as_bytes(),
            );
            self.send(&sync_packet);
        } else {
            let sync_packet = Packet::new(
                self.id,
                probe_payload.node_id,
                PacketType::Sync,
                SyncPayload::new(
                    self.id,
                    self.key_pair.public_key().as_ref().try_into().unwrap(),
                    0,
                    [0; 32],
                    0,
                )
                .as_bytes(),
            );
            self.send(&sync_packet);
        }
    }

    fn handle_sync(&mut self, packet: &Packet) {
        let sync_payload = SyncPayload::from_bytes(&packet.payload);
        self.peer_public_keys
            .insert(sync_payload.node_id, sync_payload.public_key.to_vec());

        system_log(
            self.id,
            format!(
                "Added public key of {:?}: {}",
                sync_payload.node_id,
                hex_string(&sync_payload.public_key)
            ),
        );

        if self.chain.len() < sync_payload.chain_height as usize {
            let get_chain_packet =
                Packet::new(self.id, sync_payload.node_id, PacketType::GetChain, vec![]);
            self.send(&get_chain_packet);
        }
    }

    fn handle_set_data(&mut self, packet: &Packet) {
        let data_payload = DataPayload::from_bytes(&packet.payload);
        self.cache
            .set(data_payload.name.as_str(), data_payload.data.as_ref());
        system_log(
            self.id,
            format!(
                "Cached data {:?} ({:?} bytes)",
                data_payload.name,
                data_payload.data.len()
            ),
        );

        let txn = Transaction::new(self.id, packet.src, data_payload.name, CacheOperation::Set);
        let sig = self.sign(txn.as_bytes());
        let txn_payload = TransactionPayload::new(txn.clone(), sig.as_ref().try_into().unwrap());
        self.pending_transactions.insert(txn.hash, txn.clone());

        let peers: Vec<u16> = self.peer_public_keys.keys().copied().collect();
        for peer in peers {
            let txn_packet = Packet::new(
                self.id,
                peer,
                PacketType::Transaction,
                txn_payload.as_bytes(),
            );

            self.send(&txn_packet);
        }
    }

    fn handle_get_data(&mut self, packet: &Packet) {
        let data_payload = DataPayload::from_bytes(&packet.payload);
        self.cache.get(data_payload.name.as_str());
    }

    fn handle_chain(&mut self, packet: &Packet) {
        let chain_payload = ChainPayload::from_bytes(&packet.payload);
        system_log(
            self.id,
            format!(
                "Received chain of {} blocks from {:?}",
                chain_payload.chain.len(),
                packet.src
            ),
        );
        self.chain = chain_payload.chain;
    }

    fn handle_get_chain(&mut self, packet: &Packet) {
        let chain_payload = ChainPayload::new(self.chain.clone());
        system_log(
            self.id,
            format!(
                "Sending chain of {} blocks to {:?}",
                chain_payload.chain.len(),
                packet.src
            ),
        );
        let chain_packet = Packet::new(
            self.id,
            packet.src,
            PacketType::Chain,
            chain_payload.as_bytes(),
        );
        self.send(&chain_packet);
    }

    fn handle_transaction(&mut self, packet: &Packet) {
        let transaction_payload = TransactionPayload::from_bytes(&packet.payload);
        let Some(public_key) = self.peer_public_keys.get(&packet.src) else {
            system_log(
                self.id,
                format!("No public key found for node {}", packet.src),
            );
            return;
        };
        let signature = transaction_payload.signature;
        if self.verify(
            transaction_payload.transaction.as_bytes().as_ref(),
            signature.as_ref(),
            public_key,
        ) {
            system_log(
                self.id,
                format!(
                    "Transaction verified: client {} {:?} data {:?} at node {} on {}",
                    transaction_payload.transaction.client_id,
                    transaction_payload.transaction.operation,
                    transaction_payload.transaction.data_name,
                    transaction_payload.transaction.node_id,
                    transaction_payload.transaction.timestamp
                ),
            );
            self.pending_transactions.insert(
                transaction_payload.transaction.hash,
                transaction_payload.transaction,
            );
        } else {
            system_log(
                self.id,
                format!(
                    "Transaction verification failed: client {} {:?} data {:?} at node {} on {}",
                    transaction_payload.transaction.client_id,
                    transaction_payload.transaction.operation,
                    transaction_payload.transaction.data_name,
                    transaction_payload.transaction.node_id,
                    transaction_payload.transaction.timestamp
                ),
            );
        }
    }

    fn handle_block(&mut self, packet: &Packet) {
        let block_payload = BlockPayload::from_bytes(&packet.payload);
        system_log(
            self.id,
            format!(
                "Received block #{:?} {}",
                self.chain.len(),
                hex_string(&block_payload.block.merkle_root)
            ),
        );
        self.chain.push(block_payload.block);
    }

    fn send(&mut self, packet: &Packet) {
        network_log(
            self.id,
            format!(
                "Sending {:?}-0x{:X} to {:?}",
                PacketType::from(packet.packet_type),
                packet.packet_id,
                packet.dst
            ),
        );

        if let Some(dst_addr) = self.addr_table.get(&packet.dst) {
            self.socket.send_to(&packet.as_bytes(), dst_addr).unwrap();
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
                    network_log(
                        self.id,
                        format!(
                            "*Packet* Retransmitting {:?}-0x{:X} (attempt {})",
                            PacketType::from(packet.packet_type),
                            packet_id,
                            retries + 2
                        ),
                    );
                    self.socket.send_to(&packet.as_bytes(), dst_addr).unwrap();
                    self.pending_acks
                        .insert(packet_id, (retries + 1, Instant::now(), packet));
                }
            } else {
                network_log(
                    self.id,
                    format!(
                        "*Packet* {:?}-0x{:X} failed after 3 retries",
                        PacketType::from(packet.packet_type),
                        packet_id
                    ),
                );
            }
        }
    }
}

fn hex_string(bytes: &[u8]) -> String {
    bytes
        .iter()
        .map(|b| format!("{:02x}", b))
        .collect::<String>()
}
