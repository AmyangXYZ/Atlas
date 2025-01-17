use std::{
    net::{SocketAddr, UdpSocket},
    time::Duration,
};

pub trait Port {
    fn send(&self, dst: &str, data: &[u8]) -> Option<usize>;
    fn receive(&self, buffer: &mut [u8]) -> Option<(usize, SocketAddr)>;
}

pub struct UdpPort(UdpSocket);

impl UdpPort {
    pub fn bind(addr: &str, timeout: Duration) -> Option<Self> {
        let socket = UdpSocket::bind(addr).ok()?;
        socket.set_read_timeout(Some(timeout)).ok()?;
        Some(Self(socket))
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
