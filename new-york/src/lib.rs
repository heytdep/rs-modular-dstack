//! This is an experimental implementation. It is fairly minimal and has more trust assumptions than dstack-vm because it posts
//! and holds less data on-chain.
//! 
//! The idea here is to explore a more minimal and fully onchain approach to orchestration. Registering only posts a quote on-chain
//! instead than relying on a subscription service. The quote is then picked up and validated by the nodes which post the encrypted
//! shared secret. The only thing this implementaion will be checking against is probably that the secret corresponds to the public key 
//! likely set as an env variable. We also infer at start time if the cluster contract was bootstrapped or not. 
//! 

use blake2::{Blake2s256, Digest};
use dstack_core::{GuestServiceInner, GuestServiceInnerCryptoHelper, HostServiceInner, TdxOnlyGuestServiceInner};

use async_trait::async_trait;
use x25519_dalek::EphemeralSecret;
mod stellar;

// TODO change types depending on the chain we're posting to.
pub struct HostServices {
    pub contract: [u8; 32],
    pub secret: [u8; 32],
}

impl HostServices {
    pub fn new(contract: [u8; 32], secret: [u8; 32]) -> Self {
        Self {
            contract,
            secret
        }
    }
}

#[async_trait]
impl HostServiceInner for HostServices {
    type Pubkey = [u8; 32];
    type Quote = String;
    type Signature = Vec<u8>;

    async fn bootstrap(&self, quote: Self::Quote, pubkeys: Vec<Self::Pubkey>) -> anyhow::Result<()> {
        let shared_pubkey = pubkeys[0]; 
        stellar::post_bootstrap(self.contract, self.secret, quote, shared_pubkey).await?;
        
        Ok(())
    }

    async fn register(
        &self,
        quote: Self::Quote,
        _pubkeys: Vec<Self::Pubkey>,
        _signatures: Vec<Self::Signature>,
    ) -> anyhow::Result<()> {
        stellar::post_register(self.contract, self.secret, quote).await?;

        Ok(())
    }
}

pub struct GuestServices {}

impl GuestServices {
    pub fn new() -> Self {
        Self {  }
    }
}

#[async_trait]
impl GuestServiceInner for GuestServices {
    type Pubkey = [u8; 32];
    type EncryptedMessage = Vec<u8>;

    async fn replicate_thread(&self) -> anyhow::Result<()> {
        Ok(())
    }

    async fn onboard_new_node(&self, quote: Self::Quote, pubkeys: Vec<Self::Pubkey>) -> anyhow::Result<Self::EncryptedMessage> {
        Ok(vec![])
    }
}

#[async_trait]
impl TdxOnlyGuestServiceInner for GuestServices {
    type Tag = String;
    type DerivedKey = [u8; 32];

    // Note: likely used for sealing.
    async fn get_derived_key(&self, tag: Self::Tag) -> anyhow::Result<Self::DerivedKey> {
        let mut hasher = Blake2s256::new();
        Ok([0; 32])
    }
}

#[async_trait]
impl GuestServiceInnerCryptoHelper for GuestServices {
    type Appdata = Vec<u8>;
    type Quote = Vec<u8>;
    type Keypair = (x25519_dalek::PublicKey, x25519_dalek::EphemeralSecret);
    type VerificationResult = serde_json::Value;

    async fn get_quote(&self, appdata: Self::Appdata) -> anyhow::Result<Self::Quote> {
        Ok(vec![])
    }

    async fn verify_quote(&self, quote: Self::Quote) -> anyhow::Result<Self::VerificationResult> {
        Ok(serde_json::Value::Null)
    }
    
    fn get_keypair(&self) -> anyhow::Result<(Self::Keypair)> {
        let secret = EphemeralSecret::random();
        let pubkey = x25519_dalek::PublicKey::from(&secret);

        Ok((pubkey, secret))
    }
}
