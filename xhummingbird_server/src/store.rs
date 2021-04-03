use crate::protos::event::Event;
use crate::helper;
use std::collections::BTreeMap;

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
        let time = helper::timestamp_u64(&event);

        self.data.insert(time, event);
    }

    pub fn head(&self, from: Option<u64>, title: Option<String>) -> Vec<&Event>{
        let iter = match from {
            None => self.data.range(..).rev(),
            Some(u) => self.data.range(..u).rev(),
        };

        let mut events = Vec::new();
        let mut n = 0;

        for event in iter {
            let event = event.1;

            if title == None || event.title == title.as_deref().unwrap() {
                events.push(event);

                n += 1;

                if n > 10 {
                    break;
                }
            }
        }

        events
    }
}
