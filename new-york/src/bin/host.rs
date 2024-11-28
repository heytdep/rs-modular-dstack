use std::sync::Arc;

use dstack_core::host_paths;
use new_york::HostServices;
use warp::Filter;

#[tokio::main]
async fn main() {
    let host_internal = HostServices::new([0;32], [0;32]);
    let host_paths = host_paths::HostPaths::new(Arc::new(host_internal));

    warp::serve(host_paths.bootstrap().or(host_paths.register())).run(([127,0,0,1], 8000)).await;
}