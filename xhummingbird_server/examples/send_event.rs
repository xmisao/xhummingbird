use protobuf::well_known_types::Timestamp;
use protobuf::Message;
use protobuf::RepeatedField;
use std::collections::HashMap;
use std::convert::TryFrom;
use std::thread;
use std::time::{Duration, SystemTime};
use xhummingbird_server::protos::event::Event;

fn main() {
    println!("send_event started.");

    let context = zmq::Context::new();
    let publisher = context.socket(zmq::PUSH).unwrap();
    assert!(publisher.connect("tcp://localhost:8800").is_ok());

    // Send events to localhost 100 times
    for n in 0..10 {
        let event = build_sample_event(n);
        println!("{:?}", event);
        let bytes = event.write_to_bytes().unwrap();

        publisher.send(&bytes, 0).unwrap();
        thread::sleep(Duration::from_millis(1000));
    }

    println!("send_event finished.");

    thread::park();
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
