use atlas::node::Node;

fn main() {
    let node = Node::new(1, "127.0.0.1:8080");
    println!("Node created: {}", node.id);
    let message = "hello world";
    let sig = node.sign(message.to_string());
    println!("Signature: {:0x?}", sig.as_ref());
    let verified = node.verify(message.to_string(), sig);
    println!("Verification: {}", verified);
    node.serve();
}
