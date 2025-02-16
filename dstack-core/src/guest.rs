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
    type Pubkey: Send + Sync + DeserializeOwned + Serialize;
    type EncryptedMessage: Send + Sync + Serialize;
    type Quote: Send + Sync + DeserializeOwned;
    type SharedKey;

    async fn get_secret(&self) -> anyhow::Result<Self::SharedKey>;

    async fn replicate_thread(&self) -> anyhow::Result<()>;

    async fn onboard_new_node(
        &self,
        quote: Self::Quote,
        pubkeys: Vec<Self::Pubkey>,
    ) -> anyhow::Result<Self::EncryptedMessage>;
}

#[async_trait]
pub trait TdxOnlyGuestServiceInner {
    type Tag: Send + Sync + DeserializeOwned;
    type DerivedKey: Send + Sync + Serialize;

    /// Note: tag here is not necessarily. string since we want to allow for more
    /// customizability around them e.g have structured tag objects.
    async fn get_derived_key(&self, tag: Self::Tag) -> anyhow::Result<Self::DerivedKey>;
}
