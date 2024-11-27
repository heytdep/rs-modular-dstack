//! This is an experimental implementation. It is fairly minimal and has more trust assumptions than dstack-vm because it posts
//! and holds less data on-chain.
//! 

use dstack_core::HostServiceInner;

use async_trait::async_trait;
mod stellar;

// TODO change types depending on the chain we're posting to.
pub struct HostServices {
    pub contract: [u8; 32],
    pub secret: [u8; 32],
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
