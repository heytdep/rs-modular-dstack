#![cfg(test)]

use super::*;
use soroban_sdk::{Env, String};

#[test]
fn test() {
    let env = Env::default();
    let contract_id = env.register_contract(None, ClusterContract);
    let client = ClusterContractClient::new(&env, &contract_id);

    client.bootstrap(
        &String::from_str(&env, "bootstrap"),
        &String::from_str(&env, "quote"),
    );
    client.register(
        &String::from_str(&env, "register"),
        &String::from_str(&env, "quote"),
    );
    client.onboard(
        &String::from_str(&env, "onboard"),
        &String::from_str(&env, "encrypted_shared_secret"),
    );
}
