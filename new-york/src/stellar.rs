//! Example of using the Stellar network as comms layer for our new-york implementation.
//!
//! Note that this example uses the Mercury API and ZVM program to construct transactions before signing them as a proof of concept
//! for development ease. If availability is a primary concern, then the host should also be running at least a watcher
//! node to both fetch the events and submitting transactions (and simulation should also be local).
//!

mod utils;

use anyhow::anyhow;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::json;
use stellar_xdr::curr::{Limits, ReadXdr, Transaction};
use utils::sign_transaction;
//use x25519_dalek::{EphemeralSecret, PublicKey};

#[derive(Serialize, Deserialize, Clone)]
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

#[derive(Serialize, Deserialize, Clone)]
pub struct PendingObject {
    // hex-encoded.
    pub quote: String,
    pub pubkey: String,
}

pub async fn post_bootstrap(
    cluster_contract: [u8; 32],
    secret_key: [u8; 32],
    quote: String,
    shared_pubkey: [u8; 32],
) -> anyhow::Result<()> {
    let cluster_contract = stellar_strkey::Contract(cluster_contract).to_string();
    let zephyr_url = "https://api.mercurydata.app/zephyr/execute/39";
    let payload = json!({
        "project_name": "newyork",
        "mode": {
            "Function": {
                "fname": "bootstrap",
                "arguments": format!(r#"{{
                    \"cluster\": \"{}\",
                    \"pubkey\": \"{}\",
                    \"quote\": \"{}\",
            }}"#, cluster_contract, hex::encode(shared_pubkey), quote)
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

    if let Some(envelope) = txenvelope.envelope {
        sign_and_send_tx(envelope, secret_key).await?
    }

    Ok(())
}

pub async fn post_register(
    cluster_contract: [u8; 32],
    secret_key: [u8; 32],
    quote: String,
    node_pubkey: &[u8; 32],
) -> anyhow::Result<()> {
    let cluster_contract = stellar_strkey::Contract(cluster_contract).to_string();
    let zephyr_url = "https://api.mercurydata.app/zephyr/execute/39";
    let payload = json!({
        "project_name": "newyork",
        "mode": {
            "Function": {
                "fname": "register",
                "arguments": format!(r#"{{
                    \"cluster\": \"{}\",
                    \"quote\": \"{}\",
                    \"pubkey\": \"{}\",
            }}"#, cluster_contract, quote, hex::encode(node_pubkey))
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

    if let Some(envelope) = txenvelope.envelope {
        sign_and_send_tx(envelope, secret_key).await?
    }

    Ok(())
}

pub async fn get_pending(cluster_contract: [u8; 32]) -> anyhow::Result<Vec<PendingObject>> {
    let cluster_contract = stellar_strkey::Contract(cluster_contract).to_string();
    let zephyr_url = "https://api.mercurydata.app/zephyr/execute/39";
    let payload = json!({
        "project_name": "newyork",
        "mode": {
            "Function": {
                "fname": "pending",
                "arguments": format!(r#"{{
                        \"cluster\": \"{}\",
                    }}"#, cluster_contract)
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

pub async fn get_onboarded(
    cluster_contract: [u8; 32],
    node_pubkey: &[u8; 32],
) -> anyhow::Result<String> {
    let cluster_contract = stellar_strkey::Contract(cluster_contract).to_string();
    let zephyr_url = "https://api.mercurydata.app/zephyr/execute/39";
    let payload = json!({
        "project_name": "newyork",
        "mode": {
            "Function": {
                "fname": "onboarded",
                "arguments": format!(r#"{{
                    \"cluster\": \"{}\",
            }}"#, cluster_contract)
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
    let onboarded: Vec<OnboardedObject> = response.json().await?;
    for onboarded in onboarded {
        if onboarded.node_pubkey == hex::encode(node_pubkey) {
            return Ok(onboarded.shared_secret);
        }
    }

    Err(anyhow!("").into())
}

pub async fn sign_and_send_tx(envelope: String, secret_key: [u8; 32]) -> anyhow::Result<()> {
    let stellar_secret_key = stellar_strkey::ed25519::PrivateKey(secret_key).to_string();

    let tx = Transaction::from_xdr_base64(envelope.clone(), Limits::none());
    let signed = sign_transaction(
        tx.unwrap(),
        "Test SDF Network ; September 2015",
        &stellar_secret_key,
    );

    let response = reqwest::Client::new()
        .post(format!("https://horizon-testnet.stellar.org/transactions"))
        .header("Content-Type", "application/x-www-form-urlencoded")
        .body(format!("tx={}", urlencoding::encode(&signed)))
        .send()
        .await?
        .text()
        .await?;

    println!("Executed transaction, response: {}\n", response);

    Ok(())
}
