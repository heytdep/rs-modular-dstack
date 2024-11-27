use std::sync::Arc;
use warp::{reject::Rejection, reply::WithStatus, Filter};
use super::HostServiceInner;

pub(crate) fn with_impl<H>(
    host_internal: Arc<H>,
) -> impl Filter<Extract = (Arc<H>,), Error = std::convert::Infallible> + Clone
where
    H: HostServiceInner + Sync + Send,
{
    warp::any().map(move || host_internal.clone())
}

pub struct HostPaths<H: HostServiceInner> {
    pub inner_host: Arc<H>,
}

mod requests {
    use serde::Deserialize;

    use crate::HostServiceInner;

    #[derive(Deserialize)]
    pub struct BootstrapArgs<H: HostServiceInner> {
        pub quote: H::Quote,
        pub pubkeys: Vec<H::Pubkey>,
    }

    #[derive(Deserialize)]
    pub struct RegisterArgs<H: HostServiceInner> {
        pub quote: H::Quote,
        pub pubkeys: Vec<H::Pubkey>,
        pub signatures: Vec<H::Signature>,
    }
}

// TODO: better response handling.

impl<H: HostServiceInner + Send + Sync> HostPaths<H> {
    pub fn new(host_internal: Arc<H>) -> Self {
        Self {
            inner_host: host_internal,
        }
    }

    pub fn bootstrap(
        &self,
    ) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
        warp::path("bootstrap")
            .and(warp::post())
            .and(warp::body::json())
            .and(with_impl(self.inner_host.clone()))
            .and_then(
                |request: requests::BootstrapArgs<H>, host_impl: Arc<H>| async move {
                    match host_impl.bootstrap(request.quote, request.pubkeys).await {
                        Ok(_) => {
                            return Ok::<WithStatus<String>, Rejection>(warp::reply::with_status(
                                "success".into(),
                                warp::http::StatusCode::CREATED,
                            ))
                        }
                        Err(e) => {
                            return Ok(warp::reply::with_status(
                                format!("error while bootstrapping in inner host impl {:?}", e),
                                warp::http::StatusCode::INTERNAL_SERVER_ERROR,
                            ))
                        }
                    }
                },
            )
    }

    pub fn register(
        &self,
    ) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
        warp::path("register")
            .and(warp::post())
            .and(warp::body::json())
            .and(with_impl(self.inner_host.clone()))
            .and_then(
                |request: requests::RegisterArgs<H>, host_impl: Arc<H>| async move {
                    match host_impl.register(request.quote, request.pubkeys, request.signatures).await {
                        Ok(_) => {
                            return Ok::<WithStatus<String>, Rejection>(warp::reply::with_status(
                                "success".into(),
                                warp::http::StatusCode::CREATED,
                            ))
                        }
                        Err(e) => {
                            return Ok(warp::reply::with_status(
                                format!("error while registering in inner host impl {:?}", e),
                                warp::http::StatusCode::INTERNAL_SERVER_ERROR,
                            ))
                        }
                    }
                },
            )
    }
}
