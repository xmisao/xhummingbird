use xhummingbird_server::protos::event::Event;
use protobuf::Message;
use protobuf::RepeatedField;
use std::collections::HashMap;
use std::fs::File;
use std::io::prelude::*;
use protobuf::well_known_types::Timestamp;
use std::time::SystemTime;
use std::convert::TryFrom;

fn main(){
    println!("write_event started.");

    let mut event = Event::new();
    event.set_level(1);
    event.set_title("SampleEvent".to_string());
    event.set_message("Something happend".to_string());

    let trace = RepeatedField::from_vec(
        vec!(
            "trace 1".to_string(),
            "trace 2".to_string(),
            "trace 3".to_string(),
        )
    );
    event.set_trace(trace);

    let mut tags = HashMap::new();
    tags.insert("key".to_string(), "value".to_string());
    event.set_tags(tags);

    let mut timestamp = Timestamp::new();

    let since = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap();
    let sec = since.as_secs();
    let nsec = since.subsec_nanos();

    timestamp.set_seconds(TryFrom::try_from(sec).unwrap());
    timestamp.set_nanos(TryFrom::try_from(nsec).unwrap());
    event.set_timestamp(timestamp);

    println!("{:?}", event);

    // Write event to event.bin
    let mut file = File::create("event.bin").unwrap();
    let bytes = event.write_to_bytes().unwrap();
    file.write(&bytes).unwrap();

    println!("write_event finished.");
}
