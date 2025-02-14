use std::time::Duration;

use async_trait::async_trait;
use dstack_core::InnerAttestationHelper;
use sha2::{Digest, Sha256};

pub struct Attestation {}

impl Attestation {
    pub fn new() -> Self {
        Self {}
    }
}

#[async_trait]
impl InnerAttestationHelper for Attestation {
    type Appdata = Vec<u8>;
    type Quote = String;
    type VerificationResult = dcap_qvl::verify::VerifiedReport;

    async fn get_quote(&self, appdata: Self::Appdata) -> anyhow::Result<Self::Quote> {
        let preimage = format!("register{}", hex::encode(appdata));
        let mut hasher = Sha256::new();
        hasher.update(preimage);
        let hashed: Vec<u8> = hasher.finalize().to_vec();
        let mut padded_report_data = [0_u8; 64];
        padded_report_data[..hashed.len()].copy_from_slice(&hashed);

        let (_, quote) = tdx_attest::get_quote(&padded_report_data, None)?;
        let hex_quote = hex::encode(quote);
        println!("Hex Output: {}", hex_quote);
        
        Ok(hex_quote)
    }

    async fn verify_quote(&self, quote: Self::Quote) -> anyhow::Result<Self::VerificationResult> {
        let quote = hex::decode(quote)?;
        
        // we're just relying on intel's API. We can change whenever we want.
        let collateral = dcap_qvl::collateral::get_collateral_from_pcs(&quote, Duration::from_secs(15)).await?;
        let now = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs();
        let tcb = dcap_qvl::verify::verify(&quote, &collateral, now).map_err(|e| anyhow::anyhow!(e as u32))?;

        Ok(tcb)
    }
}
