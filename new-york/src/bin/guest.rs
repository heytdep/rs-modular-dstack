use std::{env, sync::Arc};

use dstack_core::{guest_paths, GuestServiceInner};
use new_york::GuestServices;
use warp::Filter;

// Note: as you'll notice, the pattern for setting the secret is really bad, will have to find a good way to deal
// with inferring the secret. A solution which I'm not a fan of would be to wrap in a mutex/rwlock
// (mutex probs better here).

#[tokio::main]
async fn main() {
    let cluster_string = env::var("CLUSTER").unwrap();
    let cluster_contract = stellar_strkey::Contract::from_string(&cluster_string)
        .unwrap()
        .0;

    let mut guest_internal_replication = GuestServices::new(cluster_contract);
    let mut with_shared_secret = GuestServices::new(cluster_contract);

    // if operator infers PUBKEY then we want to join an already-bootstrapped cluster.
    // else we want to be bootstrapping the cluster ourselves (replay protection should be onchain).
    if let Ok(expected_shared_pubkey) = env::var("PUBKEY") {
        let bytes = hex::decode(expected_shared_pubkey)
            .unwrap()
            .try_into()
            .unwrap();
        guest_internal_replication.set_expected_public(bytes);
        with_shared_secret.set_expected_public(bytes);
    }

    let threadsafe = Arc::new(guest_internal_replication);
    let secret = threadsafe.replicate_thread().await;

    with_shared_secret.set_secret(secret.unwrap());
    let threadsafe = Arc::new(with_shared_secret);
    let guest_paths: guest_paths::GuestPaths<GuestServices> =
        guest_paths::GuestPaths::new(threadsafe);

    let _ = tokio::join!(
        warp::serve(guest_paths.onboard_new_node().or(guest_paths.status()))
            .run(([0, 0, 0, 0], 3030)),
        // Note: this is currently unsafe, this microservice should probably only run within
        // the podman container and fed with the shared secret as environment variable.
        warp::serve(guest_paths.get_derived_key()).run(([127, 0, 0, 1], 3031))
    );
}
