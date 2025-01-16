use std::{thread, time::Duration};

use atlas::{node::Node, orchestrator::Orchestrator};

fn main() {
    let orchestrator = thread::spawn(move || {
        let mut orchestrator = Orchestrator::new("127.0.0.1:8080");
        orchestrator.run();
    });
    let node1 = thread::spawn(move || {
        let mut node1 = Node::new(1, "127.0.0.1:8081");
        thread::sleep(Duration::from_secs(1));
        node1.join("127.0.0.1:8080");
        node1.run();
    });
    // let node2 = thread::spawn(move || {
    //     let node2 = Node::new(2, "127.0.0.1:8082");
    //     let message = "hello world";
    //     let sig = node2.sign(message.to_string());
    //     node2.send("127.0.0.1:8080", node2.public_key().as_ref());
    //     thread::sleep(Duration::from_secs(1));
    //     node2.send("127.0.0.1:8080", sig.as_ref());
    // });

    orchestrator.join().unwrap();
    node1.join().unwrap();
    // node2.join().unwrap();
}
