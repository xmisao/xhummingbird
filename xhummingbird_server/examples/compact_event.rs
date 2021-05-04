use protobuf::well_known_types::Timestamp;
use protobuf::Message;
use protobuf::RepeatedField;
use std::collections::HashMap;
use std::convert::TryFrom;
use std::thread;
use std::rc::Rc;
use std::time::{Duration, SystemTime};
use xhummingbird_server::protos::event::Event;
use std::collections::hash_map::DefaultHasher;
use std::hash::Hasher;
use std::hash::Hash;
use std::ops::Deref;

fn main(){
    let mut compactor = Compactor::new();

    let mut compacted_events = Vec::new();

    for n in 0..10 {
        let event = build_sample_event(n);
        let compacted_event = compactor.compact_event(&event);
        println!("{:?}", compacted_event);
        compacted_events.push(compacted_event);
    }

    for compacted_event in &compacted_events {
        let event = compacted_event.uncompact_event();
        println!("{:?}", event);
    }

    println!("{:?}", compactor);
}

fn build_sample_event(n: u32) -> Event {
    let mut event = Event::new();
    event.set_level(1);
    event.set_title("SampleEvent".to_string());
    event.set_message(format!("Something happend #{}", n));

    let trace = RepeatedField::from_vec(vec![
        "trace 1".to_string(),
        "trace 2".to_string(),
        "trace 3".to_string(),
    ]);
    event.set_trace(trace);

    let mut tags = HashMap::new();
    tags.insert("key".to_string(), "value".to_string());
    event.set_tags(tags);

    let mut timestamp = Timestamp::new();

    let since = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap();
    let sec = since.as_secs();
    let nsec = since.subsec_nanos();

    timestamp.set_seconds(TryFrom::try_from(sec).unwrap());
    timestamp.set_nanos(TryFrom::try_from(nsec).unwrap());
    event.set_timestamp(timestamp);

    event
}

#[derive(Debug)]
struct Compactor {
    strings: HashMap<u64, Rc<String>>,
}

impl Compactor {
    fn new() -> Compactor {
        let strings = HashMap::new();

        Compactor{
            strings,
        }
    }

    fn compact_str(&mut self, str: &String) -> Rc<String>{
        let mut hasher = DefaultHasher::new();
        str.hash(&mut hasher);
        let hash = hasher.finish();

        match self.strings.get(&hash) {
            Some(shared_string) => shared_string.clone(),
            _ => {
                let shared_string = Rc::new(str.clone());
                self.strings.insert(hash, shared_string.clone());
                shared_string
            },
        }
    }

    fn compact_event(&mut self, event: &Event) -> CompactedEvent {
        let level = event.level;

        let title = self.compact_str(&event.title);
        let message = self.compact_str(&event.message);

        let mut trace = Vec::new();
        for line in &event.trace {
            trace.push(self.compact_str(&line));
        }

        let mut tags = HashMap::new();
        for (k, v) in &event.tags {
            let k = self.compact_str(&k);
            let v = self.compact_str(&v);
            tags.insert(k, v);
        }

        let timestamp = event.timestamp.clone();

        CompactedEvent{level, title, message, trace, tags, timestamp}
    }
}

#[derive(Debug)]
struct CompactedEvent {
    pub level: u32,
    pub title: Rc<String>,
    pub message: Rc<String>,
    pub trace: Vec<Rc<String>>,
    pub tags: HashMap<Rc<String>, Rc<String>>,
    pub timestamp: ::protobuf::SingularPtrField<::protobuf::well_known_types::Timestamp>,
}

impl CompactedEvent {
    fn uncompact_event(&self) -> Event {
        let mut event = Event::new();

        event.set_level(self.level);
        event.set_title(self.title.deref().clone());
        event.set_message(self.message.deref().clone());
        let mut trace = Vec::new();
        for v in &self.trace {
            trace.push(v.deref().clone());
        }
        event.set_trace(RepeatedField::from_vec(trace));
        let mut tags = HashMap::new();
        for (k, v) in &self.tags {
            tags.insert(k.deref().clone(), v.deref().clone());
        }
        event.set_tags(tags);
        event.set_timestamp(self.timestamp.clone().unwrap());

        event
    }
}
