//! Home for host service module interface. The goal here is to offer a hardcoded set of paths the host service
//! relies on, while allowing the network implementor to define how they want the workflow for those paths to look
//! like. For example, the network hardcodes a path to post the join request to the chain, but it's up to the implementor
//! to decide how to post that request (this specifically allows to e.g easily support multiple base chains).
//!
//! The reasoning behind this structure is to provide a well-defined path for developers to build dstack implementations
//! while giving them power to shape the actual functionality.

use async_trait::async_trait;
use serde::{de::DeserializeOwned, Serialize};

pub mod paths;

#[async_trait]
pub trait HostServiceInner {
    type Quote: DeserializeOwned + Serialize + Send + Sync;
    type Pubkey: DeserializeOwned + Serialize + Send + Sync;
    type Signature: DeserializeOwned + Serialize + Send + Sync;

    /// Performs the registering request.
    ///
    /// [`self`] is for global state (e.g orchestrator contract id, secret key used to submit transactions, etc).
    /// [`quote`] is the quote generated from the crafted appdata.
    /// [`pubkeys`] is a vector of public keys (addresses and/or pubkeys if you're familiar with amiller/dstack-vm). The idea
    /// is that each TDX implementor will want to have different ways and layers for working with signatures various public keys.
    /// [`signatures`] is a vector of signatures also passed from the TDX. Reasoning is the same as the above.
    ///
    /// For example, the amiller/dstack-vm impl includes two pubkeys in the appdata (pubkey of privkey and myaddr of myPriv). mypriv signs
    /// the register_appdata for the host address and uses it as sig to pass to the host verification. The pubkey on the other hand
    /// does not immediately sign anything but it's the pubkey that the existing nodes encrypt the shared secret to.  Using this design
    /// implementors can choose to use a different approach. For instance here's a more minimalistic heavily chain-based approach:
    /// (new-node side): generate a secret, add the public key to the report, request the quote and post it directly on-chain (just post the quote).
    /// (existing-node side): listen for new quotes, validate them and encyrpt the shared secret using the public key in the quote + post it.
    async fn register(
        &self,
        quote: Self::Quote,
        pubkeys: Vec<Self::Pubkey>,
        signatures: Vec<Self::Signature>,
    ) -> anyhow::Result<()>;

    /// Handles the actual creation of a cluster contract (i.e a contract configured with the pubkey of the shared secret).
    ///
    /// [`self`] is for global state (e.g orchestrator contract id, secret key used to submit transactions, etc).
    /// [`quote`] is the "genesis" quote. No one has to check against this quote, but it shuold be audited from other nodes that intend
    /// to join before they actually join the quorum
    async fn bootstrap(&self, quote: Self::Quote, pubkeys: Vec<Self::Pubkey>)
        -> anyhow::Result<()>;

    async fn onboard_thread(&self) -> anyhow::Result<()>;
}

#[async_trait]
pub trait HostServiceInnerCryptoHelper: HostServiceInner {
    type VerificationResult;

    async fn verify_quote(&self, quote: Self::Quote) -> anyhow::Result<Self::VerificationResult>;
}
