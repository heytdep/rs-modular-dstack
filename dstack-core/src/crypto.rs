use async_trait::async_trait;

#[async_trait]
pub trait InnerCryptoHelper {
    // TBD: are these the best constraints? Likely not.
    // TBD: make everything async?? Likely not
    type Appdata: Send + Sync;
    type Quote: Send + Sync;
    type Keypair: Send + Sync;
    type VerificationResult;

    async fn get_quote(&self, appdata: Self::Appdata) -> anyhow::Result<Self::Quote>;

    /// Getting a new (likely random) keypair
    fn get_keypair(&self) -> anyhow::Result<Self::Keypair>;

    async fn verify_quote(&self, quote: Self::Quote) -> anyhow::Result<Self::VerificationResult>;
}
