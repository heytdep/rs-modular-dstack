use serde::{Deserialize, Serialize};
use zephyr_sdk::{
    prelude::*,
    soroban_sdk::{String as SorobanString, Symbol, TryIntoVal},
    DatabaseDerive, DatabaseInteract, EnvClient, TransactionResponse,
};

#[derive(DatabaseDerive, Clone, Serialize)]
#[with_name("pending")]
pub struct Pending {
    pub pubkey: String,
    pub quote: String,
}

#[derive(DatabaseDerive, Clone, Serialize)]
#[with_name("onboard")]
pub struct Onboard {
    pub pubkey: String,
    pub encrypted: String,
}

#[no_mangle]
pub extern "C" fn on_close() {
    let env = EnvClient::new();
    let events = env.reader().pretty().soroban_events();
    for event in events {
        if stellar_strkey::Contract(event.contract).to_string() == env!("CLUSTER") {
            let topic1: Symbol = env.from_scval(&event.topics.to_vec()[0]);
            let pubkey: SorobanString = env.from_scval(&event.topics.to_vec()[0]); // note: we always have a topic2 for simple-cluster so this is safe

            if topic1 == Symbol::new(&env.soroban(), "register") {
                let quote: SorobanString = env.from_scval(&event.data);
                let mut slice = Vec::new();
                quote.copy_into_slice(&mut slice);
                let quote_string = String::from_utf8(slice.clone()).unwrap();
                slice.clear();
                pubkey.copy_into_slice(&mut slice);

                let new_pending = Pending {
                    quote: quote_string,
                    pubkey: String::from_utf8(slice).unwrap(),
                };
                new_pending.put(&env);
            } else if topic1 == Symbol::new(&env.soroban(), "onboard") {
                let encrypted: SorobanString = env.from_scval(&event.data);
                let mut slice = Vec::new();
                encrypted.copy_into_slice(&mut slice);
                let encrypted_string = String::from_utf8(slice.clone()).unwrap();
                slice.clear();
                pubkey.copy_into_slice(&mut slice);

                
                let new_onboard = Onboard {
                    encrypted: encrypted_string,
                    pubkey: String::from_utf8(slice).unwrap(),
                };
            }
        }
    }
}

//
// TX BUILDERS
//

#[derive(Deserialize)]
pub struct PostArgs {
    source: String,
    cluster: String,
    quote: String,
    pubkey: String,
}

#[derive(Deserialize)]
pub struct GetArgs {
    cluster: String,
}

fn get_sequence(env: &EnvClient, source: &str) -> i64 {
    let account = stellar_strkey::ed25519::PublicKey::from_string(source)
        .unwrap()
        .0;

    env.read_account_from_ledger(account)
        .unwrap()
        .unwrap()
        .seq_num as i64
        + 1
}

fn simulate_contract_call(env: &EnvClient, body: &PostArgs, function_name: &str) -> TransactionResponse {
    let sequence = get_sequence(env, &body.source);
    let quote = SorobanString::from_str(&env.soroban(), &body.quote);
    let pubkey = SorobanString::from_str(&env.soroban(), &body.pubkey);

    env.simulate_contract_call_to_tx(
        body.source.clone(),
        sequence,
        stellar_strkey::Contract::from_string(&body.cluster)
            .unwrap()
            .0,
        Symbol::new(&env.soroban(), function_name),
        (pubkey, quote).try_into_val(env.soroban()).unwrap(),
    ).unwrap()
}

#[no_mangle]
pub extern "C" fn bootstrap() {
    let env = EnvClient::empty();
    let body: PostArgs = env.read_request_body();
    let result = simulate_contract_call(&env, &body, "bootstrap");
    
    env.conclude(result);
}

#[no_mangle]
pub extern "C" fn onboard() {
    let env = EnvClient::empty();
    let body: PostArgs = env.read_request_body();
    let result = simulate_contract_call(&env, &body, "onboard");

    env.conclude(result);
}

#[no_mangle]
pub extern "C" fn register() {
    let env = EnvClient::empty();
    let body: PostArgs = env.read_request_body();
    let result = simulate_contract_call(&env, &body, "register");

    env.conclude(result);
}

//
// READERS
//

#[no_mangle]
pub extern "C" fn pending() {
    let env = EnvClient::empty();

    let pending: Vec<Pending> = env.read();
    env.conclude(&pending);
}

#[no_mangle]
pub extern "C" fn onboarded() {
    let env = EnvClient::empty();

    let onboard: Vec<Onboard> = env.read();
    env.conclude(&onboard);
}
