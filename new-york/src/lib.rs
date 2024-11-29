//! This is an experimental implementation. It is fairly minimal and has more trust assumptions than dstack-vm because it posts
//! and holds less data on-chain.
//!
//! The idea here is to explore a more minimal and fully onchain approach to orchestration. Registering only posts a quote on-chain
//! instead than relying on a subscription service. The quote is then picked up and validated by the nodes which post the encrypted
//! shared secret. The only thing this implementaion will be checking against is probably that the secret corresponds to the public key
//! likely set as an env variable. We also infer at start time if the cluster contract was bootstrapped or not.
//!
use anyhow::anyhow;
use async_trait::async_trait;
use diffie_hellman::Crypto;
use dstack_core::{
    guest_paths, host_paths, GuestServiceInner, HostServiceInner, InnerAttestationHelper,
    InnerCryptoHelper, TdxOnlyGuestServiceInner,
};
use dummy_attestation::Attestation;
use sha2::{Digest, Sha256};
use std::time::Duration;
use stellar::get_onboarded;
use tokio::time::sleep;
use x25519_dalek::StaticSecret;

mod stellar;

// NOTE: just for ease.
const NONCE: [u8; 32] = [0; 32];

// TODO change types depending on the chain we're posting to.
pub struct HostServices {
    pub contract: [u8; 32],
    pub secret: [u8; 32],
}

impl HostServices {
    pub fn new(contract: [u8; 32], secret: [u8; 32]) -> Self {
        Self { contract, secret }
    }
}

#[async_trait]
impl HostServiceInner for HostServices {
    type Pubkey = [u8; 32];
    type Quote = String;
    type Signature = Vec<u8>;

    async fn bootstrap(
        &self,
        quote: Self::Quote,
        pubkeys: Vec<Self::Pubkey>,
    ) -> anyhow::Result<()> {
        let shared_pubkey = pubkeys[0];
        stellar::post_bootstrap(self.contract, self.secret, quote, shared_pubkey).await?;

        Ok(())
    }

    async fn register(
        &self,
        quote: Self::Quote,
        pubkeys: Vec<Self::Pubkey>,
        _signatures: Vec<Self::Signature>,
    ) -> anyhow::Result<()> {
        stellar::post_register(self.contract, self.secret, quote, &pubkeys[0]).await?;

        Ok(())
    }

    async fn onboard_thread(&self) -> anyhow::Result<()> {
        println!("Onboarding thread started");
        loop {
            println!("Checking for new onboard requests ...");
            if let Ok(current_pending) = stellar::get_pending(self.contract).await {
                for pending in current_pending {
                    let quote = pending.quote;
                    let pubkey = pending.pubkey;
                    let pubkey_bytes = hex::decode(&pubkey)?.try_into().unwrap();

                    // call tdx host-facing interface.
                    let client = reqwest::Client::new();
                    let resp = client
                        .post("http://localhost:3030/onboard")
                        .json(&guest_paths::requests::OnboardArgs::<GuestServices> {
                            quote,
                            pubkeys: vec![pubkey_bytes],
                        })
                        .send()
                        .await?;
                    let message: <GuestServices as GuestServiceInner>::EncryptedMessage =
                        resp.json().await?;
                    println!(
                        "Onboarding {} with encrypted message {}",
                        pubkey,
                        hex::encode(&message)
                    );
                    stellar::post_onboard(self.contract, self.secret, message, &pubkey_bytes)
                        .await?;
                }
            }

            sleep(Duration::from_secs(5)).await
        }
    }
}

pub struct GuestServices {
    // Implementor's configs including helper objects.
    cluster_contract: [u8; 32],
    shared_public: Option<[u8; 32]>,
    shared_secret: Option<[u8; 32]>,
    attestation: Attestation,
    crypto: Crypto,
}

impl GuestServices {
    pub fn new(cluster_contract: [u8; 32]) -> Self {
        Self {
            cluster_contract,
            shared_public: None,
            shared_secret: None,
            attestation: Attestation::new(),
            crypto: Crypto::new(),
        }
    }

    pub fn set_expected_public(&mut self, public: [u8; 32]) {
        self.shared_public = Some(public)
    }

    pub fn set_secret(&mut self, secret: [u8; 32]) {
        self.shared_secret = Some(secret)
    }
}

#[async_trait]
impl GuestServiceInner for GuestServices {
    type Pubkey = [u8; 32];
    type EncryptedMessage = Vec<u8>;
    type SharedKey = [u8; 32];
    type Quote = String;

    // Note: the implementor decides for themselves how they want the secret to be stored in
    // [`self`]
    fn get_secret(&self) -> anyhow::Result<Self::SharedKey> {
        if let Some(shared) = self.shared_secret {
            Ok(shared)
        } else {
            Err(anyhow!("").into())
        }
    }

    async fn replicate_thread(&self) -> anyhow::Result<Self::SharedKey> {
        println!("Replicating ...");
        let client = reqwest::Client::new();

        let (my_pubkey, my_secret) = self.crypto.get_keypair()?;
        let quote = self
            .attestation
            .get_quote(my_pubkey.as_bytes().to_vec())
            .await?;
        let mut shared_secret;

        // Note: whether to bootstrap is operator inferred not chain-inferred.
        if let Some(expected_shared_pubkey_bytes) = self.shared_public {
            // We need to register
            let request_onboard = client
                .post("http://localhost:8000/register")
                .json(&host_paths::requests::RegisterArgs::<HostServices> {
                    quote: hex::encode(quote),
                    pubkeys: vec![my_pubkey.as_bytes().clone()],
                    signatures: vec![],
                })
                .send()
                .await?
                .text()
                .await?;
            println!("Got response {}", request_onboard);
            loop {
                if let Ok(encrypted_encoded) =
                    get_onboarded(self.cluster_contract, my_pubkey.as_bytes()).await
                {
                    // NOTE: this is bad rn because any malicious user can spam the comms network and
                    // send invalid shared keys to prevent new nodes from joining. This is easily avoidable
                    // with some extra code. It might also be good to abstract the public key checking.
                    println!("Found encrypted message for this node, processing ...");
                    let encrypted_raw = hex::decode(encrypted_encoded)?;
                    let decrypted = self.crypto.decrypt_secret(
                        NONCE,
                        encrypted_raw,
                        vec![expected_shared_pubkey_bytes.into()],
                        vec![my_secret.clone()],
                    )?;
                    let shared_secret_bytes = decrypted.as_bytes();
                    // note: we don't need to explicitly check the obtained shared secret because thanks to diffie
                    // hellman constraints + TDX and replication guarantees (if the encrypted secret was not signed with the shared secret
                    // then the decoding would fail due to a diff in the p2p shared secret, if it was signed by the secret
                    // we know that it was a cluster-trusted TD so we know the message is indeed the encrypted shared secret).
                    shared_secret = *shared_secret_bytes;
                    break;
                } else {
                    println!("Didn't hear from cluster contract yet, waiting 5 seconds");
                    sleep(Duration::from_secs(5)).await;
                }
            }
        } else {
            // We need to bootstrap
            let request_bootstrap = client
                .post("http://localhost:8000/bootstrap")
                .json(&host_paths::requests::BootstrapArgs::<HostServices> {
                    quote: hex::encode(quote),
                    pubkeys: vec![my_pubkey.as_bytes().clone()],
                })
                .send()
                .await?
                .text()
                .await?;
            println!(
                "Bootstrapping cluster contract with shared public key {}: {}",
                hex::encode(my_pubkey.as_bytes()),
                request_bootstrap
            );
            shared_secret = *my_secret.as_bytes();
        }
        println!("Got secret! {}", hex::encode(shared_secret));
        Ok(shared_secret)
    }

    /// Verifies the provided quote ensuring that [`pubkeys[0]`] is within the quote, if that
    /// succeeds (i.e secretkey is held only in tdx) then it encrypts the shared secret to
    /// [`pubkeys[0]`].
    async fn onboard_new_node(
        &self,
        quote: Self::Quote,
        pubkeys: Vec<Self::Pubkey>,
    ) -> anyhow::Result<Self::EncryptedMessage> {
        let verify = self.attestation.verify_quote(quote).await?;

        let expected_appdata: [u8; 32] = {
            let preimage = format!("register{}", hex::encode(pubkeys[0].to_vec()));
            let mut hasher = Sha256::new();
            hasher.update(preimage);
            hasher.finalize().into()
        };
        let got_appdata = verify.get_appdata();

        if expected_appdata != got_appdata {
            return Err(anyhow!("").into());
        }

        let encrypted = self.crypto.encrypt_secret(
            NONCE,
            self.shared_secret.ok_or(anyhow!(""))?.into(),
            pubkeys.iter().map(|p| (*p).into()).collect(),
        )?;
        Ok(encrypted)
    }
}

/// NON host-facing paths here.
#[async_trait]
impl TdxOnlyGuestServiceInner for GuestServices {
    type Tag = String;
    type DerivedKey = String;

    async fn get_derived_key(&self, tag: Self::Tag) -> anyhow::Result<Self::DerivedKey> {
        let mut hasher = Sha256::new();
        hasher.update(format!(
            "{}{}",
            tag,
            hex::encode(self.shared_secret.ok_or(anyhow!(""))?)
        ));
        let derived = hasher.finalize();

        Ok(hex::encode(derived))
    }
}

#[cfg(test)]
mod test;
