use crate::protos::event::Event;
use crate::helper;
use std::collections::{BTreeMap, HashMap};
use protobuf::Message;
use std::convert::TryFrom;
use std::fs::File;
use std::io::{self, Write, BufWriter};
use chrono::{Utc, TimeZone, Duration};
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

                if n > 100 {
                    break;
                }
            }
        }

        events
    }

    pub fn stat(&self, title: Option<String>) -> Vec<u64>{
        let from_dt = chrono::Utc::now().checked_sub_signed(Duration::hours(168)).unwrap();
        let sec:u64 = from_dt.timestamp().try_into().unwrap();
        let nsec:u64 = from_dt.timestamp_subsec_nanos().try_into().unwrap();
        let from:u64 = sec * 1_000_000_000 + nsec;

        let iter = self.data.range(from..).rev();

        let mut stat = Vec::new();
        for _ in 0..168 {
            stat.push(0);
        }

        for event in iter {
            let event = event.1;

            if title == None || event.title == title.as_deref().unwrap() {
                let event_time = helper::timestamp_u64(event);

                let event_timestamp = helper::timestamp_u64(event);
                let index = (event_timestamp - from) / (60 * 60 * 1_000_000_000);

                if index < 168 {
                    stat[index as usize] += 1;
                }
            }
        }

        stat
    }

    pub fn titles(&self) -> HashMap<String, u64>{
        let from_dt = chrono::Utc::now().checked_sub_signed(Duration::hours(168)).unwrap();
        let sec:u64 = from_dt.timestamp().try_into().unwrap();
        let nsec:u64 = from_dt.timestamp_subsec_nanos().try_into().unwrap();
        let from:u64 = sec * 1_000_000_000 + nsec;

        let iter = self.data.range(from..).rev();

        let mut titles = HashMap::new();

        for event in iter {
            let event = event.1;

            let v = titles.entry(event.title.clone()).or_insert_with(||{0});
            *v += 1;
        }

        titles
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
