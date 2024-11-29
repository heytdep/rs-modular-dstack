use async_trait::async_trait;
use dcap_quotes::QuoteVerificationResult;
use dstack_core::InnerAttestationHelper;
use reqwest::Client;
use sha2::{Digest, Sha256};

pub struct Attestation {}

impl Attestation {
    pub fn new() -> Self {
        Self {}
    }
}

/// Dummy attestation helpers. This should be moved to a default and either be derived or implemented
/// in a wrapped object.
#[async_trait]
impl InnerAttestationHelper for Attestation {
    type Appdata = Vec<u8>;
    type Quote = String;
    type VerificationResult = QuoteVerificationResult;

    async fn get_quote(&self, appdata: Self::Appdata) -> anyhow::Result<Self::Quote> {
        let preimage = format!("register{}", hex::encode(appdata));
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
            .await?
            .bytes()
            .await?;

        let hex_quote = hex::encode(response);
        println!("Hex Output: {}", hex_quote);

        Ok(hex_quote)
    }

    async fn verify_quote(&self, quote: Self::Quote) -> anyhow::Result<Self::VerificationResult> {
        let quote = hex::decode(quote)?;
        let client = Client::new();

        let verification_resp = client
            .post("http://ns31695324.ip-141-94-163.eu:10080/verify")
            .header("Content-Type", "application/octet-stream")
            .body(quote)
            .send()
            .await?;

        Ok(verification_resp.json().await?)
    }
}
