use std::{env, sync::Arc};

use dstack_core::{guest_paths, GuestServiceInner};
use new_york::GuestServices;
use warp::Filter;

// Note: as you'll notice, the pattern for setting the secret is really bad, will have to find a good way to deal
// with inferring the secret. A solution which I'm not a fan of would be to wrap in a mutex/rwlock
// (mutex probs better here).

#[tokio::main]
async fn main() {
    // NB: depending on what your requirements around measurements are you might need to hardcode these as build vars.
    let cluster_string = env::var("CLUSTER").unwrap();
    let maybe_expected_shared_pubkey = env::var("PUBKEY");
    
    let cluster_contract = stellar_strkey::Contract::from_string(&cluster_string)
        .unwrap()
        .0;

    let mut guest_internal = GuestServices::new(cluster_contract);

    // if operator infers PUBKEY then we want to join an already-bootstrapped cluster.
    // else we want to be bootstrapping the cluster ourselves (replay protection should be onchain).
    if let Ok(expected_shared_pubkey) = maybe_expected_shared_pubkey {
        let bytes = hex::decode(expected_shared_pubkey)
            .unwrap()
            .try_into()
            .unwrap();

        guest_internal.set_expected_public(bytes).await;
    }

    let threadsafe = Arc::new(guest_internal);
    let replication_reference = threadsafe.clone();
    
    let handle_replication = tokio::spawn(async move {
        replication_reference.replicate_thread().await
    });

    let guest_paths: guest_paths::GuestPaths<GuestServices> =
        guest_paths::GuestPaths::new(threadsafe);

    let _ = tokio::join!(
        handle_replication,
        warp::serve(
            guest_paths.onboard_new_node()
            .or(guest_paths.status())
            // NB: this endpoint is sensitive since it allows anyone who can reach it to construct a valid shared key. 
            // It's important the implementor makes sure that this connection is only available within the deployed pod. 
            // This allows for the quote to hold the measurements of the expected pod config and prevents new pods or 
            // the host environment to retrieve the shared secret.
            .or(guest_paths.get_derived_key()))
            .run(([0, 0, 0, 0], 3030)),
    );
}
