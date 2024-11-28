//! Home for the guest services module interface. 
//! 
//! Note: I'm just prototyping here but the goal is to have a dedicated trait for
//! cryptography helpers (keypair generation and quote verification) to offer default implementations
//! that will likely be more used as opposed to the actual cluster interaction flow. Not
//! sure we actually need this in core fwiw.
//! 
//! Note: currently self is used to have args that are inferred by the implementor at
//! object declaration.
//! 

pub mod paths;

use async_trait::async_trait;
use serde::{de::DeserializeOwned, Serialize};

#[async_trait]
pub trait GuestServiceInner: TdxOnlyGuestServiceInner {
    type Pubkey: Send + Sync + DeserializeOwned;
    type EncryptedMessage: Send + Sync + Serialize;

    async fn replicate_thread(&self) -> anyhow::Result<()>;

    async fn onboard_new_node(&self, quote: Self::Quote, pubkeys: Vec<Self::Pubkey>) -> anyhow::Result<Self::EncryptedMessage>;
}

#[async_trait]
pub trait TdxOnlyGuestServiceInner: GuestServiceInnerCryptoHelper {
    type Tag: Send + Sync + DeserializeOwned;
    type DerivedKey: Send + Sync;

    /// Note: tag here is not necessarily. string since we want to allow for more
    /// customizability around them e.g have structured tag objects.
    async fn get_derived_key(&self, tag: Self::Tag) -> anyhow::Result<Self::DerivedKey>;
}

#[async_trait]
pub trait GuestServiceInnerCryptoHelper {
    // TBD: are these the best constraints? Likely not.
    // TBD: make everything async?? Likely not
    type Appdata: Send + Sync;
    type Quote: Send + Sync + DeserializeOwned;
    type Keypair: Send + Sync;
    type VerificationResult;

    async fn get_quote(&self, appdata: Self::Appdata) -> anyhow::Result<Self::Quote>;

    async fn verify_quote(&self, quote: Self::Quote) -> anyhow::Result<Self::VerificationResult>;

    /// Getting a new (likely random) keypair
    fn get_keypair(&self) -> anyhow::Result<Self::Keypair>;
}