use std::{thread, time::Duration};

use atlas::{
    client::Client,
    node::{Node, ATLAS_PORT},
};

fn main() {
    let orchestrator = thread::spawn(move || {
        let mut orchestrator = Node::new(0, "127.0.0.1");
        orchestrator.run();
    });

    let node1 = thread::spawn(move || {
        let mut node1 = Node::new(1, "127.0.0.2");
        thread::sleep(Duration::from_secs(1));
        node1.run();
    });

    let client = thread::spawn(move || {
        let mut client = Client::new(
            2,
            Duration::from_secs(1),
            format!("127.0.0.1:{}", ATLAS_PORT).as_str(),
        );
        let mut sat = 0;

        loop {
            thread::sleep(Duration::from_secs(1));
            client.set_data(&format!("/satellite/{}", sat % 10), "world".as_bytes());
            thread::sleep(Duration::from_secs(1));
            client.get_data(&format!("/satellite/{}", sat % 10));
            sat += 1;
        }
    });

    orchestrator.join().expect("Orchestrator thread failed");
    node1.join().expect("Node 1 thread failed");
    client.join().expect("Client thread failed");
}
