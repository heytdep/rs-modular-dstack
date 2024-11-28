use std::sync::Arc;

use dstack_core::guest_paths;
use new_york::GuestServices;

#[tokio::main]
async fn main() {
    let host_internal = GuestServices::new();
    let guest_paths: guest_paths::GuestPaths<GuestServices> = guest_paths::GuestPaths::new(Arc::new(host_internal));

    tokio::join!(
        warp::serve(guest_paths.onboard_new_node()).run(([127,0,0,1], 3030)),
        
        // Note: this is currently unsafe, this microservice should only run within
        // the podman container and fed with the shared secret as environment variable.
        warp::serve(guest_paths.get_derived_key()).run(([127,0,0,1], 3031))
    );
}