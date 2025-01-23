use std::{thread, time::Duration};

use atlas::{client::Client, node::Node};

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
        let mut client = Client::new(2, Duration::from_secs(1), "127.0.0.1");
        loop {
            thread::sleep(Duration::from_secs(5));
            client.set_data("hello", "world".as_bytes());
            thread::sleep(Duration::from_secs(5));
            client.get_data("hello");
        }
    });

    orchestrator.join().expect("Orchestrator thread failed");
    node1.join().expect("Node 1 thread failed");
    client.join().expect("Client thread failed");
}
