use super::handlers;
use crate::{scheduled_actions::Actions, ReadState};
use backend::message;
use crossbeam::channel::Sender;
use std::convert::Infallible;
use warp::Filter;

pub fn get(
    state: ReadState,
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    warp::path!("state")
        .and(warp::get())
        .and(with_state(state))
        .and_then(handlers::get)
}

pub fn receive(
    sender: Sender<Actions>,
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    warp::path("receive")
        .and(warp::post())
        .and(json_body())
        .and(with_sender(sender))
        .and_then(handlers::receive)
}

fn with_state(state: ReadState) -> impl Filter<Extract = (ReadState,), Error = Infallible> + Clone {
    warp::any().map(move || state.clone())
}

fn with_sender(
    sender: Sender<Actions>,
) -> impl Filter<Extract = (Sender<Actions>,), Error = Infallible> + Clone {
    warp::any().map(move || sender.clone())
}

fn json_body() -> impl Filter<Extract = (message::Message,), Error = warp::Rejection> + Clone {
    warp::body::json()
}
