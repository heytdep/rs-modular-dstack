//! Example of using the Stellar network as comms layer for our new-york implementation.
//!
//! Note that this example uses the Mercury API and ZVM program to construct transactions before signing them as a proof of concept
//! for development ease. If availability is a primary concern, then the host should also be running at least a watcher
//! node to both fetch the events and submitting transactions (and simulation should also be local).
//!

mod utils;

use anyhow::anyhow;
use ed25519_dalek::SigningKey;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::json;
use utils::sign_and_send_tx;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct TransactionResponse {
    pub status: Option<String>,
    pub envelope: Option<String>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct OnboardedObject {
    // hex-encoded.
    pub node_pubkey: String,
    pub shared_secret: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct PendingObject {
    // hex-encoded.
    pub quote: String,
    pub pubkey: String,
}

async fn post_to_zephyr(
    secret_key: [u8; 32],
    function_name: &str,
    args: String,
) -> anyhow::Result<()> {
    let zephyr_url = "https://api.mercurydata.app/zephyr/execute/113";
    let payload = json!({
        "project_name": "newyork",
        "mode": {
            "Function": {
                "fname": function_name,
                "arguments": args,
            }
        }
    });

    let client = Client::new();
    let response = client
        .post(zephyr_url)
        .header("Content-Type", "application/json")
        .json(&payload)
        .send()
        .await?;
    let txenvelope: TransactionResponse = response.json().await?;
    println!("Got transaction envelope to sign: {:?}", txenvelope);

    if let Some(envelope) = txenvelope.envelope {
        sign_and_send_tx(envelope, secret_key).await?
    }

    Ok(())
}

// This won't post anything to be pulled client side for automated replication but
// ensures that the cluster contract is controlled by the allegedly TDX-generated shared pubkey.
// Again, this is a minimal dstack implementation, so the nodes have to audit the cluster before
// joining it, i.e they need to make sure that the shared pubkey is within the valid TDX quote.
pub async fn post_bootstrap(
    cluster_contract: [u8; 32],
    secret_key: [u8; 32],
    quote: String,
    shared_pubkey: [u8; 32],
) -> anyhow::Result<()> {
    let public = stellar_strkey::ed25519::PublicKey(
        *SigningKey::from_bytes(&secret_key)
            .verifying_key()
            .as_bytes(),
    )
    .to_string();

    let args = json!({
        "cluster": stellar_strkey::Contract(cluster_contract).to_string(),
        "pubkey": hex::encode(shared_pubkey),
        "quote": quote,
        "source": public
    }).to_string();

    post_to_zephyr(secret_key, "bootstrap", args).await
}

// This will post new data to get_pending allowing the onboard thread to get the quotes + pubkeys
// of the nodes that want to join the cluster.
pub async fn post_register(
    cluster_contract: [u8; 32],
    secret_key: [u8; 32],
    quote: String,
    node_pubkey: &[u8; 32],
) -> anyhow::Result<()> {
    let public = stellar_strkey::ed25519::PublicKey(
        *SigningKey::from_bytes(&secret_key)
            .verifying_key()
            .as_bytes(),
    )
    .to_string();

    let args = json!({
        "cluster": stellar_strkey::Contract(cluster_contract).to_string(),
        "quote": quote,
        "pubkey": hex::encode(node_pubkey),
        "source": public
    }).to_string();

    post_to_zephyr(secret_key, "register", args).await
}

// This will post new data to get_onboard allowing the replicatoor to get the encrypted message.
pub async fn post_onboard(
    cluster_contract: [u8; 32],
    secret_key: [u8; 32],
    encrypted_message: Vec<u8>,
    node_pubkey: &[u8; 32],
) -> anyhow::Result<()> {
    let public = stellar_strkey::ed25519::PublicKey(
        *SigningKey::from_bytes(&secret_key)
            .verifying_key()
            .as_bytes(),
    )
    .to_string();

    let args = json!({
        "cluster": stellar_strkey::Contract(cluster_contract).to_string(),
        "encrypted": hex::encode(encrypted_message),
        "pubkey": hex::encode(node_pubkey),
        "source": public
    }).to_string();

    post_to_zephyr(secret_key, "onboard", args).await
}

async fn pull_from_zephyr<T: serde::de::DeserializeOwned>(
    cluster_contract: [u8; 32],
    function_name: &str,
) -> anyhow::Result<T> {
    let cluster_contract = stellar_strkey::Contract(cluster_contract).to_string();
    let zephyr_url = "https://api.mercurydata.app/zephyr/execute/113";
    let payload = json!({
        "project_name": "newyork",
        "mode": {
            "Function": {
                "fname": function_name,
                "arguments": format!(r#"{{\"cluster\": \"{}\"}}"#, cluster_contract),
            }
        }
    });

    let client = Client::new();
    let response = client
        .post(zephyr_url)
        .header("Content-Type", "application/json")
        .json(&payload)
        .send()
        .await?;

    Ok(response.json().await?)
}

pub async fn get_pending(cluster_contract: [u8; 32]) -> anyhow::Result<Vec<PendingObject>> {
    pull_from_zephyr(cluster_contract, "pending").await
}

pub async fn get_onboarded(
    cluster_contract: [u8; 32],
    node_pubkey: &[u8; 32],
) -> anyhow::Result<String> {
    let onboarded: Vec<OnboardedObject> = pull_from_zephyr(cluster_contract, "onboarded").await?;
    for onboarded in onboarded {
        if onboarded.node_pubkey == hex::encode(node_pubkey) {
            return Ok(onboarded.shared_secret);
        }
    }

    Err(anyhow!("No matching onboarded node found").into())
}
