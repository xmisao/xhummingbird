use crate::protos::event::Event;
use crate::helper;
use std::collections::BTreeMap;
use protobuf::Message;
use std::convert::TryFrom;
use std::fs::File;
use std::io::{self, Write, BufWriter};

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

    pub fn get(&self, id: u64) -> Option<&Event>{
        self.data.get(&id)
    }

    pub fn save(&self, path: &str) -> Result<usize, io::Error> {
        let file = File::create(path).unwrap();
        let mut writer = BufWriter::new(file);
        let mut n = 0;

        for (_, event) in &self.data {
            let bytes = event.write_to_bytes().unwrap();
            let size:u32 = TryFrom::try_from(bytes.len()).unwrap();

            writer.write_all(&size.to_ne_bytes());
            writer.write_all(&bytes);

            n += 1;
        }

        let zero: u32 = 0;
        writer.write_all(&zero.to_ne_bytes());

        writer.flush().unwrap();

        Ok(n)
    }
}
