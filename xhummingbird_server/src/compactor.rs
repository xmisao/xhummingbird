use std::collections::HashMap;
use std::collections::hash_map::DefaultHasher;
use std::hash::Hasher;
use std::hash::Hash;
use std::ops::Deref;
use std::rc::Rc;
use crate::protos::event::Event;
use protobuf::RepeatedField;

#[derive(Debug)]
pub struct Compactor {
    strings: HashMap<u64, Rc<String>>,
}

impl Compactor {
    pub fn new() -> Compactor {
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

    pub fn compact_event(&mut self, event: &Event) -> CompactedEvent {
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
pub struct CompactedEvent {
    pub level: u32,
    pub title: Rc<String>,
    pub message: Rc<String>,
    pub trace: Vec<Rc<String>>,
    pub tags: HashMap<Rc<String>, Rc<String>>,
    pub timestamp: ::protobuf::SingularPtrField<::protobuf::well_known_types::Timestamp>,
}

impl CompactedEvent {
    pub fn uncompact_event(&self) -> Event {
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
