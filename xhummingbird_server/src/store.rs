use crate::protos::event::Event;
use std::collections::BTreeMap;
use std::convert::TryInto;

pub struct Store {
    data: BTreeMap<u64, Event>
}

impl Store {
    pub fn new() -> Store{
        Store{
            data: BTreeMap::new()
        }
    }

    pub fn put(&mut self, event: Event){
        let nsec:u64 = event.get_timestamp().get_nanos().try_into().unwrap();
        let sec:u64 = event.get_timestamp().get_seconds().try_into().unwrap();
        let time = sec * 1_000_000_000 + nsec;

        self.data.insert(time, event);
    }

    pub fn head(&self) -> Vec<&Event>{
        self.data.iter().rev().take(10).map(|t| t.1).collect()
    }
}
