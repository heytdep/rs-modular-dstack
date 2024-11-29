//! Example minimal cluster contract.
//!
//! This contract is nothing more than a comms layer for nodes and a store for the shared public key. More enshrined
//! implementations may want to add additional parameters to the store as well as contact logic to verify signatures, etc.
//!

#![no_std]
use soroban_sdk::{contract, contractimpl, contracttype, symbol_short, Env, String};

#[contract]
pub struct ClusterContract;

#[contracttype]
pub enum DataKey {
    SharedPub,
}

#[contractimpl]
impl ClusterContract {
    pub fn bootstrap(env: Env, shared_public: String, quote: String) {
        if env.storage().instance().has(&DataKey::SharedPub) {
            panic!() // already bootstrapped
        }
        env.storage()
            .instance()
            .set(&DataKey::SharedPub, &shared_public);
        env.events()
            .publish((symbol_short!("boot"), shared_public), quote);
    }

    pub fn register(env: Env, node_pubkey: String, quote: String) {
        if !env.storage().instance().has(&DataKey::SharedPub) {
            panic!() // not bootstrapped
        }

        env.events()
            .publish((symbol_short!("register"), node_pubkey), quote);
    }

    // Note: anyone can call this!
    pub fn onboard(env: Env, node_pubkey: String, encrypted: String) {
        if !env.storage().instance().has(&DataKey::SharedPub) {
            panic!() // not bootstrapped
        }

        env.events()
            .publish((symbol_short!("onboard"), node_pubkey), encrypted);
    }
}

mod test;
