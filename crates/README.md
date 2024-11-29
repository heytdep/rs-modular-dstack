# Standard helper implementations

The `dstack-core` library expresses cartain interfaces that are not directly used in core but by the implementor. These interfaces do not need to be followed by the implementor, but exist to ease development for common patterns. For example, `crates/diffie-hellman` provides an implementation to generate keypairs, encrypt and decrypt secrets without the implementor having to worry about shared secrets or cryptographic operations:

```rust
async fn replicate_thread(&self) -> anyhow::Result<Self::SharedKey> {
    // ...
    let (my_pubkey, my_secret) = self.crypto.get_keypair()?;
    // ...
    println!("Found encrypted message for this node, processing ...");
    let encrypted_raw = hex::decode(encrypted_encoded)?;
    let decrypted = self.crypto.decrypt_secret(
        NONCE,
        encrypted_raw,
        vec![expected_shared_pubkey_bytes.into()],
        vec![my_secret.clone()],
    )?;
    let shared_secret_bytes = decrypted.as_bytes();
    // NOTE: this is bad rn because any malicious user can spam the comms network and
    // send invalid shared keys to prevent new nodes from joining. This is easily avoidable
    // with some extra code. It might also be good to abstract the public key checking.
    let shared_pubkey =
        x25519_dalek::PublicKey::from(&StaticSecret::from(*shared_secret_bytes));
    if &expected_shared_pubkey_bytes != shared_pubkey.as_bytes() {
        panic!("Nodes posted invalid shared secret")
    }
    // ...
}
```

Following the same interface allows other implementations (e.g using a different secret sharing method) can be automatically swapped without any effort on the implementor's end.

There are other ways to obtain this reusable behaviour, but as an MVP this works good enough.
