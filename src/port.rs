use std::net::{SocketAddr, UdpSocket};

pub trait Port {
    fn send(&self, dst: &str, data: &[u8]) -> Option<usize>;
    fn receive(&self, buffer: &mut [u8]) -> Option<(usize, SocketAddr)>;
}

pub struct UdpPort(UdpSocket);

impl UdpPort {
    pub fn bind(addr: &str) -> Option<Self> {
        UdpSocket::bind(addr).ok().map(Self)
    }
}

impl Port for UdpPort {
    fn send(&self, dst: &str, data: &[u8]) -> Option<usize> {
        self.0.send_to(data, dst).ok()
    }

    fn receive(&self, buffer: &mut [u8]) -> Option<(usize, SocketAddr)> {
        self.0.recv_from(buffer).ok()
    }
}
