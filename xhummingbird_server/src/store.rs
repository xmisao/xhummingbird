use crate::compactor::*;
use crate::helper;
use crate::messages::EventSummary;
use crate::protos::event::Event;
use chrono::{Duration, DateTime, NaiveDateTime, Utc};
use protobuf::Message;
use std::collections::{BTreeMap, HashMap};
use std::convert::TryFrom;
use std::convert::TryInto;
use std::fs::File;
use std::io::{self, BufWriter, Write};
use std::ops::Deref;

pub struct Store {
    data: BTreeMap<u64, CompactedEvent>,
    compactor: Compactor,
}

impl Store {
    pub fn new() -> Store {
        Store {
            data: BTreeMap::new(),
            compactor: Compactor::new(),
        }
    }

    pub fn put(&mut self, event: Event) {
        let time = helper::timestamp_u64(&event);

        self.data.insert(time, self.compactor.compact_event(&event));
    }

    pub fn head(
        &self,
        from: Option<u64>,
        title: Option<String>,
        service: Option<String>,
    ) -> Vec<Event> {
        let iter = match from {
            None => self.data.range(..).rev(),
            Some(u) => self.data.range(..u).rev(),
        };

        let mut events = Vec::new();
        let mut n = 0;

        for event in iter {
            let event = event.1;

            if title == None || event.title.deref() == title.as_deref().unwrap() {
                if service == None || event.service.deref() == service.as_deref().unwrap() {
                    events.push(event.uncompact_event());

                    n += 1;

                    if n > 100 {
                        break;
                    }
                }
            }
        }

        events
    }

    pub fn stat(&self, title: Option<String>, service: Option<String>) -> Vec<u64> {
        let from_dt = chrono::Utc::now()
            .checked_sub_signed(Duration::hours(168))
            .unwrap();
        let sec: u64 = from_dt.timestamp().try_into().unwrap();
        let nsec: u64 = from_dt.timestamp_subsec_nanos().try_into().unwrap();
        let from: u64 = sec * 1_000_000_000 + nsec;

        let iter = self.data.range(from..).rev();

        let mut stat = Vec::new();
        for _ in 0..168 {
            stat.push(0);
        }

        for event in iter {
            let event = event.1;

            if title == None || event.title.deref() == title.as_deref().unwrap() {
                if service == None || event.service.deref() == service.as_deref().unwrap() {
                    let event_timestamp = helper::timestamp_u64(&event.uncompact_event());
                    let index = (event_timestamp - from) / (60 * 60 * 1_000_000_000);

                    if index < 168 {
                        stat[index as usize] += 1;
                    }
                }
            }
        }

        stat
    }

    pub fn titles(
        &self,
        filter_title: Option<String>,
        filter_service: Option<String>,
    ) -> Vec<EventSummary> {
        let from_dt = chrono::Utc::now()
            .checked_sub_signed(Duration::hours(168))
            .unwrap();
        let sec: u64 = from_dt.timestamp().try_into().unwrap();
        let nsec: u64 = from_dt.timestamp_subsec_nanos().try_into().unwrap();
        let from: u64 = sec * 1_000_000_000 + nsec;

        let iter = self.data.range(from..);

        let mut titles = HashMap::new();
        let mut latest_timestamps = HashMap::new();

        for event in iter {
            let event = event.1;

            let key = (event.service.deref().clone(), event.title.deref().clone());

            if !titles.contains_key(&key) {
                let mut stat = Vec::new();
                for _ in 0..42 {
                    stat.push(0);
                }

                let key = key.clone();
                titles.insert(key, stat);
            }

            let trend = titles.get_mut(&key).unwrap();

            let event_timestamp = helper::timestamp_u64(&event.uncompact_event());
            let index = (event_timestamp - from) / (60 * 60 * 1_000_000_000 * 4);
            if index < 42 {
                trend[index as usize] += 1;
            }

            latest_timestamps.insert(key, Store::convert_timestamp(&event.timestamp.clone().unwrap()));
        }

        let mut summary = Vec::new();

        for ((service, title), trend) in titles {
            if filter_title == None || filter_title.as_deref().unwrap() == title {
                if filter_service == None || filter_service.as_deref().unwrap() == service {
                    let mut count = 0;
                    for n in &trend {
                        count += n
                    }

                    let key = (service.clone(), title.clone());
                    let latest_timestamp = latest_timestamps.remove(&key).unwrap();

                    let s = EventSummary {
                        service: service.clone(),
                        title: title.clone(),
                        count,
                        trend,
                        latest_timestamp,
                    };
                    summary.push(s);
                }
            }
        }

        summary
    }

    pub fn get(&self, id: u64) -> Option<Event> {
        match self.data.get(&id) {
            Some(event) => Some(event.uncompact_event()),
            _ => None,
        }
    }

    pub fn save(&self, path: &str) -> Result<usize, io::Error> {
        let file = File::create(path).unwrap();
        let mut writer = BufWriter::new(file);
        let mut n = 0;

        for (_, event) in &self.data {
            let bytes = event.uncompact_event().write_to_bytes().unwrap();
            let size: u32 = TryFrom::try_from(bytes.len()).unwrap();

            writer.write_all(&size.to_ne_bytes()).unwrap();
            writer.write_all(&bytes).unwrap();

            n += 1;
        }

        let zero: u32 = 0;
        writer.write_all(&zero.to_ne_bytes()).unwrap();

        writer.flush().unwrap();

        Ok(n)
    }

    pub fn count(&self) -> u64 {
        self.data.len().try_into().unwrap()
    }

    fn convert_timestamp(timestamp: &protobuf::well_known_types::Timestamp) -> DateTime<Utc> {
        DateTime::<Utc>::from_utc(
            NaiveDateTime::from_timestamp(timestamp.seconds, timestamp.nanos.try_into().unwrap()),
            Utc
        )
    }
}
