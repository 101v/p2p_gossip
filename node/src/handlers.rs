use crate::{scheduled_actions::Actions, ReadState};
use backend::message::Message;
use core::result::Result;
use crossbeam::channel::Sender;
use std::convert::Infallible;

pub async fn get(state: ReadState) -> Result<impl warp::Reply, Infallible> {
    let state = state.read().unwrap();
    Ok(warp::reply::json(&*state))
}

pub async fn receive(
    message: Message,
    sender: Sender<Actions>,
) -> Result<impl warp::Reply, Infallible> {
    sender.send(Actions::Respond(message)).unwrap();
    Ok("Received")
}
