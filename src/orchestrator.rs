use std::collections::HashMap;

use crate::{
    port::{Port, UdpPort},
    protocol::{
        JoinRequestPayload, JoinResponsePayload, KeyPayload, Packet, PacketType, PACKET_BUFFER_SIZE,
    },
};

pub struct Orchestrator {
    id: u16,
    port: UdpPort,
    addr_table: HashMap<u16, String>,
    public_keys: HashMap<u16, Vec<u8>>,
}

impl Orchestrator {
    pub fn new(address: &str) -> Self {
        Self {
            id: 0,
            port: UdpPort::bind(address).unwrap(),
            addr_table: HashMap::new(),
            public_keys: HashMap::new(),
        }
    }

    pub fn run(&mut self) {
        let mut buffer = [0; PACKET_BUFFER_SIZE];
        while let Some((size, addr)) = self.port.receive(&mut buffer) {
            let Some(packet) = Packet::from_bytes(&buffer[..size]) else {
                continue;
            };
            self.addr_table.insert(packet.src, addr.to_string());
            println!("Received packet from {:?}", packet.src);

            match PacketType::from(packet.packet_type) {
                PacketType::JoinRequest => self.handle_join_request(packet),
                PacketType::KeyRequest => self.handle_key_request(packet),
                _ => continue,
            }
        }
    }

    fn handle_join_request(&mut self, packet: Packet) {
        let join_request_payload = JoinRequestPayload::from_bytes(&packet.payload);
        self.public_keys
            .insert(join_request_payload.node_id, join_request_payload.key);

        let join_response_payload = JoinResponsePayload::new(true);
        let packet = Packet::new(
            self.id,
            join_request_payload.node_id,
            PacketType::JoinResponse,
            join_response_payload.as_bytes(),
        );
        println!(
            "Sending join response to {:?}",
            join_request_payload.node_id
        );
        if let Some(addr) = self.addr_table.get(&join_request_payload.node_id) {
            self.port.send(addr, &packet.as_bytes());
        }
    }

    fn handle_key_request(&mut self, packet: Packet) {
        let key_payload = KeyPayload::from_bytes(&packet.payload);
        let key = self.public_keys.get(&key_payload.node_id).unwrap();
        let packet = Packet::new(
            self.id,
            key_payload.node_id,
            PacketType::KeyResponse,
            key.to_vec(),
        );
        if let Some(addr) = self.addr_table.get(&key_payload.node_id) {
            self.port.send(addr, &packet.as_bytes());
        }
    }
}
