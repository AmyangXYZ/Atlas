use ring::{
    rand,
    signature::{self, KeyPair},
};

fn main() {
    // Generate a key pair
    let rng = rand::SystemRandom::new();
    let pkcs8_bytes =
        signature::Ed25519KeyPair::generate_pkcs8(&rng).expect("Failed to generate key pair");
    let key_pair = signature::Ed25519KeyPair::from_pkcs8(pkcs8_bytes.as_ref())
        .expect("Failed to parse key pair");
    println!("PKCS8 bytes: {:02X?}", pkcs8_bytes.as_ref());
    // Get the public key for later verification
    let public_key_bytes = key_pair.public_key().as_ref();
    println!("Public key: {:02X?}", public_key_bytes);

    // Data to be signed
    let msg = b"hello world";

    // Sign the message
    let sig = key_pair.sign(msg);

    // Verify the signature using the public key
    let verify_key = signature::UnparsedPublicKey::new(&signature::ED25519, public_key_bytes);
    match verify_key.verify(msg, sig.as_ref()) {
        Ok(_) => {
            println!("Message length: {} bytes", msg.len());
            println!("Signature length: {} bytes", sig.as_ref().len());
            println!("Public key length: {} bytes", public_key_bytes.len());
            println!("Verification successful!");
        }
        Err(_) => println!("Verification failed!"),
    }
}
