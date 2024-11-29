use super::GuestServiceInner;
use std::sync::Arc;
use warp::{reject::Rejection, reply::Json, Filter};

pub(crate) fn with_impl<H>(
    guest_internal: Arc<H>,
) -> impl Filter<Extract = (Arc<H>,), Error = std::convert::Infallible> + Clone
where
    H: GuestServiceInner + Sync + Send,
{
    warp::any().map(move || guest_internal.clone())
}

pub struct GuestPaths<H: GuestServiceInner> {
    pub inner_guest: Arc<H>,
}

pub mod requests {
    use serde::{Deserialize, Serialize};

    use super::super::GuestServiceInner;

    #[derive(Deserialize, Serialize)]
    pub struct OnboardArgs<H: GuestServiceInner> {
        pub quote: H::Quote,
        pub pubkeys: Vec<H::Pubkey>,
    }

    #[derive(Deserialize, Serialize)]
    pub struct GetKeyArgs<H: GuestServiceInner> {
        pub tag: H::Tag,
    }
}

// TODO: better response handling.

impl<H: GuestServiceInner + Send + Sync> GuestPaths<H> {
    pub fn new(guest_internal: Arc<H>) -> Self {
        Self {
            inner_guest: guest_internal,
        }
    }

    pub fn onboard_new_node(
        &self,
    ) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
        warp::path("onboard")
            .and(warp::post())
            .and(warp::body::json())
            .and(with_impl(self.inner_guest.clone()))
            .and_then(
                |request: requests::OnboardArgs<H>, guest_impl: Arc<H>| async move {
                    match guest_impl
                        .onboard_new_node(request.quote, request.pubkeys)
                        .await
                    {
                        Ok(encrypted) => {
                            return Ok::<Json, Rejection>(warp::reply::json(&encrypted))
                        }
                        Err(e) => {
                            return Ok(warp::reply::json(&serde_json::json!({
                                "error": format!("{:?} while onbnoarding in inner guest impl", e)
                            })))
                        }
                    }
                },
            )
    }

    // Should only be callable within trusted enclaves.
    pub fn get_derived_key(
        &self,
    ) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
        warp::path!("getkey")
            .and(warp::post())
            .and(warp::body::json())
            .and(with_impl(self.inner_guest.clone()))
            .and_then(
                |request: requests::GetKeyArgs<H>, guest_impl: Arc<H>| async move {
                    match guest_impl.get_derived_key(request.tag).await {
                        Ok(derived) => {
                            return Ok::<Json, Rejection>(warp::reply::json(
                               &derived
                            ))
                        }
                        Err(e) => {
                            return Ok(warp::reply::json(&serde_json::json!({
                                "error": format!("{:?} while getting derived key in inner guest impl", e)
                            })))
                        }
                    }
                },
            )
    }
}
