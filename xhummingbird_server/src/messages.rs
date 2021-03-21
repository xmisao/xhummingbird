use actix::prelude::*;
use crate::protos::event::Event;

#[derive(Message)]
#[rtype(result = "std::result::Result<(), ()>")]
pub struct PutEvent{
    pub event: Event
}

#[derive(Message)]
#[rtype(result = "std::result::Result<(Vec<Event>), ()>")]
pub struct HeadEvents{
    pub from: Option<u64>
}

#[derive(Message)]
#[rtype(result = "std::result::Result<(), ()>")]
pub struct CommandInput{
    pub command: String
}