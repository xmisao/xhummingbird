use crate::protos::event::Event;
use std::convert::TryInto;

pub fn timestamp_u64(event: &Event) -> u64 {
    let nsec: u64 = event.get_timestamp().get_nanos().try_into().unwrap();
    let sec: u64 = event.get_timestamp().get_seconds().try_into().unwrap();

    sec * 1_000_000_000 + nsec
}
