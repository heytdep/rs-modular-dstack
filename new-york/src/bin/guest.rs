use std::{env, sync::Arc};

use dstack_core::{guest_paths, GuestServiceInner};
use new_york::GuestServices;

// Note: as you'll notice, the pattern for setting the secret is really bad, will have to find a good way to deal
// with inferring the secret. A solution which I'm not a fan of would be to wrap in a mutex/rwlock
// (mutex probs better here).

#[tokio::main]
async fn main() {
    let guest_internal = GuestServices::new([0; 32]);
    let threadsafe = Arc::new(guest_internal);
    let secret = threadsafe.replicate_thread().await;

    let mut with_shared_secret = GuestServices::new([0; 32]);
    with_shared_secret.set_secret(secret.unwrap());

    // if operator infers PUBKEY then we want to join an already-bootstrapped cluster.
    // else we want to be bootstrapping the cluster ourselves (replay protection should be onchain).
    if let Ok(expected_shared_pubkey) = env::var("PUBKEY") {
        let bytes = hex::decode(expected_shared_pubkey)
            .unwrap()
            .try_into()
            .unwrap();
        with_shared_secret.set_expected_public(bytes);
    }

    let threadsafe = Arc::new(with_shared_secret);
    let guest_paths: guest_paths::GuestPaths<GuestServices> =
        guest_paths::GuestPaths::new(threadsafe);

    let _ = tokio::join!(
        warp::serve(guest_paths.onboard_new_node()).run(([127, 0, 0, 1], 3030)),
        // Note: this is currently unsafe, this microservice should probably only run within
        // the podman container and fed with the shared secret as environment variable.
        warp::serve(guest_paths.get_derived_key()).run(([127, 0, 0, 1], 3031))
    );
}
