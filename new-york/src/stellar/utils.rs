use ed25519_dalek::{ed25519::signature::SignerMut, SigningKey, VerifyingKey};
use sha2::{Digest, Sha256};
use stellar_xdr::curr::{
    DecoratedSignature, Hash, Limits, ReadXdr, Signature, SignatureHint, Transaction,
    TransactionEnvelope, TransactionSignaturePayload, TransactionSignaturePayloadTaggedTransaction,
    TransactionV1Envelope, WriteXdr,
};

pub fn hash_transaction(
    tx: &Transaction,
    network_passphrase: &str,
) -> Result<[u8; 32], stellar_xdr::curr::Error> {
    let signature_payload = TransactionSignaturePayload {
        network_id: Hash(Sha256::digest(network_passphrase).into()),
        tagged_transaction: TransactionSignaturePayloadTaggedTransaction::Tx(tx.clone()),
    };
    Ok(Sha256::digest(signature_payload.to_xdr(Limits::none())?).into())
}

pub fn ed25519_sign(secret_key: &str, payload: &[u8]) -> (VerifyingKey, [u8; 64]) {
    let mut signing = SigningKey::from_bytes(
        &stellar_strkey::ed25519::PrivateKey::from_string(secret_key)
            .unwrap()
            .0,
    );

    (
        signing.verifying_key(),
        signing.sign(payload).to_bytes().try_into().unwrap(),
    )
}

pub fn sign_transaction(tx: Transaction, network_passphrase: &str, secret_key: &str) -> String {
    let tx_hash = hash_transaction(&tx, network_passphrase).unwrap();
    let (verifying, tx_signature) = ed25519_sign(secret_key, &tx_hash);

    let decorated_signature = DecoratedSignature {
        hint: SignatureHint(verifying.to_bytes()[28..].try_into().unwrap()),
        signature: Signature(tx_signature.try_into().unwrap()),
    };

    let envelope = TransactionEnvelope::Tx(TransactionV1Envelope {
        tx: tx.clone(),
        signatures: [decorated_signature].try_into().unwrap(),
    });

    envelope.to_xdr_base64(Limits::none()).unwrap()
}

pub async fn sign_and_send_tx(envelope: String, secret_key: [u8; 32]) -> anyhow::Result<()> {
    let stellar_secret_key = stellar_strkey::ed25519::PrivateKey(secret_key).to_string();

    println!("{}", envelope);
    let tx = Transaction::from_xdr_base64(envelope.clone(), Limits::none());
    println!("{:?}", tx);
    let signed = sign_transaction(
        tx.unwrap(),
        "Test SDF Network ; September 2015",
        &stellar_secret_key,
    );

    let response = reqwest::Client::new()
        .post(format!("https://horizon-testnet.stellar.org/transactions"))
        .header("Content-Type", "application/x-www-form-urlencoded")
        .body(format!("tx={}", urlencoding::encode(&signed)))
        .send()
        .await?
        .text()
        .await?;

    println!("Executed transaction, response: {}\n", response);

    Ok(())
}
