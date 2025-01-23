use crate::protocol::{
    AckPayload, DataPayload, Packet, PacketType, ACK_TIMEOUT, MAX_RETRIES, PACKET_BUFFER_SIZE,
};
use std::{net::UdpSocket, thread, time::Duration};

pub struct Client {
    id: u16,
    socket: UdpSocket,
    remote_addr: String,
}

impl Client {
    pub fn new(id: u16, timeout: Duration, remote_addr: &str) -> Self {
        let socket = UdpSocket::bind("0.0.0.0:0").unwrap();
        socket.set_read_timeout(Some(timeout)).unwrap();
        Self {
            id,
            socket,
            remote_addr: remote_addr.to_string(),
        }
    }

    pub fn get_data(&mut self, data_name: &str) -> Option<Vec<u8>> {
        let mut attempts = 0;
        while attempts <= MAX_RETRIES {
            if attempts > 0 {
                thread::sleep(Duration::from_millis(ACK_TIMEOUT));
            }
            let data_packet = Packet::new(
                self.id,
                0,
                PacketType::GetData,
                DataPayload::new(data_name.to_string(), vec![]).as_bytes(),
            );
            self.socket
                .send_to(&data_packet.as_bytes(), &self.remote_addr)
                .unwrap();

            let mut buffer = [0; PACKET_BUFFER_SIZE];
            let Ok((size, _)) = self.socket.recv_from(&mut buffer) else {
                attempts += 1;
                continue;
            };

            if let Some(ack_packet) = Packet::from_bytes(&buffer[..size]) {
                if PacketType::from(ack_packet.packet_type) != PacketType::Ack {
                    attempts += 1;
                    continue;
                }
            } else {
                attempts += 1;
                continue;
            }

            let Ok((size, _)) = self.socket.recv_from(&mut buffer) else {
                attempts += 1;
                continue;
            };

            if let Some(data_packet) = Packet::from_bytes(&buffer[..size]) {
                if PacketType::from(data_packet.packet_type) != PacketType::Data {
                    attempts += 1;
                    continue;
                }
                let ack_packet = Packet::new(
                    self.id,
                    0,
                    PacketType::Ack,
                    AckPayload::new(data_packet.packet_id).as_bytes(),
                );
                self.socket
                    .send_to(&ack_packet.as_bytes(), &self.remote_addr)
                    .unwrap();

                let data_payload = DataPayload::from_bytes(&data_packet.payload);
                return Some(data_payload.data);
            } else {
                attempts += 1;
                continue;
            }
        }
        None
    }

    pub fn set_data(&mut self, data_name: &str, data: &[u8]) {
        let mut attempts = 0;
        while attempts <= MAX_RETRIES {
            if attempts > 0 {
                thread::sleep(Duration::from_millis(ACK_TIMEOUT));
            }
            let data_packet = Packet::new(
                self.id,
                0,
                PacketType::SetData,
                DataPayload::new(data_name.to_string(), data.to_vec()).as_bytes(),
            );
            self.socket
                .send_to(&data_packet.as_bytes(), &self.remote_addr)
                .unwrap();

            let mut buffer = [0; PACKET_BUFFER_SIZE];
            let Ok((size, _)) = self.socket.recv_from(&mut buffer) else {
                attempts += 1;
                println!("Failed to receive ack packet");
                continue;
            };

            if let Some(ack_packet) = Packet::from_bytes(&buffer[..size]) {
                if PacketType::from(ack_packet.packet_type) != PacketType::Ack {
                    attempts += 1;
                    continue;
                }
                return;
            } else {
                attempts += 1;
                continue;
            }
        }
    }
}
