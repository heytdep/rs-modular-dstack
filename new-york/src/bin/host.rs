use std::sync::Arc;

use dstack_core::{host_paths, HostServiceInner};
use new_york::HostServices;
use warp::Filter;

#[tokio::main]
async fn main() {
    let host_internal = HostServices::new([0; 32], [0; 32]);
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
        .run(([127, 0, 0, 1], 8000))
    );
}
