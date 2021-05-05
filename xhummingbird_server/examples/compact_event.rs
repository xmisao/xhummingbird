use protobuf::well_known_types::Timestamp;
use protobuf::Message;
use protobuf::RepeatedField;
use std::collections::HashMap;
use std::convert::TryFrom;
use std::thread;
use std::time::{Duration, SystemTime};
use xhummingbird_server::compactor::*;
use xhummingbird_server::protos::event::Event;

fn main() {
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
