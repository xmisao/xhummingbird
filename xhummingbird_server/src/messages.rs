use crate::protos::event::Event;
use actix::prelude::*;
use std::collections::HashMap;
use chrono::{DateTime, Utc};

#[derive(Message)]
#[rtype(result = "std::result::Result<(), ()>")]
pub struct PutEvent {
    pub event: Event,
}

#[derive(Message)]
#[rtype(result = "std::result::Result<(Vec<Event>), ()>")]
pub struct HeadEvents {
    pub from: Option<u64>,
    pub service: Option<String>,
    pub title: Option<String>,
}

#[derive(Message)]
#[rtype(result = "std::result::Result<(), ()>")]
pub struct CommandInput {
    pub command: String,
}

#[derive(Message)]
#[rtype(result = "std::result::Result<(Event), ()>")]
pub struct GetEvent {
    pub id: u64,
}

#[derive(Message)]
#[rtype(result = "std::result::Result<usize, std::io::Error>")]
pub struct SaveSnapshot {}

#[derive(Message)]
#[rtype(result = "std::result::Result<(Vec<u64>), ()>")]
pub struct StatEvents {
    pub title: Option<String>,
    pub service: Option<String>,
}

#[derive(Message)]
#[rtype(result = "std::result::Result<Vec<EventSummary>, ()>")]
pub struct GetTitles {
    pub title: Option<String>,
    pub service: Option<String>,
}

#[derive(Debug)]
pub struct EventSummary {
    pub service: String,
    pub title: String,
    pub count: u64,
    pub trend: Vec<u64>,
    pub latest_timestamp: DateTime<Utc>,
}

#[derive(Message)]
#[rtype(result = "std::result::Result<(), ()>")]
pub struct Stop {}

#[derive(Message)]
#[rtype(result = "std::result::Result<u64, ()>")]
pub struct CountEvents {}
