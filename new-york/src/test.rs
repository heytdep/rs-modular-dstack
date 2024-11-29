use base64::prelude::*;
use dcap_quotes::QuoteVerificationResult;
use reqwest::Client;
use sha2::{Digest, Sha256};

#[tokio::test]
async fn dummy_get_quote_verify_test() {
    let preimage = format!(
        "register{}",
        hex::encode(x25519_dalek::PublicKey::from(
            &x25519_dalek::StaticSecret::random()
        ))
    );

    let mut hasher = Sha256::new();
    hasher.update(preimage);
    let hashed = hex::encode(hasher.finalize());

    let client = Client::new();
    let response = client
        .get(format!(
            "http://ns31695324.ip-141-94-163.eu:10080/attest/{}",
            hashed
        ))
        .send()
        .await
        .unwrap()
        .bytes()
        .await
        .unwrap();

    let hex_quote = hex::encode(response);
    let quote = hex::decode(hex_quote).unwrap();
    let client = Client::new();

    let verification_resp = client
        .post("http://ns31695324.ip-141-94-163.eu:10080/verify")
        .header("Content-Type", "application/octet-stream")
        .body(quote)
        .send()
        .await
        .unwrap();

    let verification_deser = verification_resp
        .json::<QuoteVerificationResult>()
        .await
        .unwrap();
    let decoded = hex::encode(
        &BASE64_STANDARD
            .decode(verification_deser.td_quote_body.report_data)
            .unwrap()[0..32],
    );
    assert_eq!(hashed, decoded);

    println!("Appdata verified successfully");
}
