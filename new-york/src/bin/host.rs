use std::{env, sync::Arc};

use dstack_core::{host_paths, HostServiceInner};
use new_york::HostServices;
use warp::Filter;

#[tokio::main]
async fn main() {
    let cluster_string = env::var("CLUSTER").unwrap();
    let cluster_contract = stellar_strkey::Contract::from_string(&cluster_string)
        .unwrap()
        .0;
    let stellar_secret_string = env::var("SECRET").unwrap();
    let stellar_secret = stellar_strkey::ed25519::PrivateKey::from_string(&stellar_secret_string)
        .unwrap()
        .0;

    let host_internal = HostServices::new(cluster_contract, stellar_secret);
    let threadsafe = Arc::new(host_internal);

    // Note: differently from the guest replicatoor thread which needs to recover the shared
    // key first and then rebuild the service with the key, object state here is just configuration
    // variables, which is not optimal but allows us to safely clone for the two execution paths.
    let host_paths = host_paths::HostPaths::new(threadsafe.clone());

    let _ = tokio::join!(
        threadsafe.onboard_thread(),
        warp::serve(
            host_paths
                .bootstrap()
                .or(host_paths.register())
                .or(host_paths.status())
        )
        .run(([0, 0, 0, 0], 8000))
    );
}
