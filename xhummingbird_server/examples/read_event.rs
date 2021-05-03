use protobuf::Message;
use std::fs;
use xhummingbird_server::protos::event::Event;

fn main() {
    println!("read_event started.");

    // Read event from event.bin
    let bytes = fs::read("event.bin").unwrap();
    let event = Event::parse_from_bytes(&bytes).unwrap();
    println!("{:?}", event);

    println!("read_event finished.");
}
