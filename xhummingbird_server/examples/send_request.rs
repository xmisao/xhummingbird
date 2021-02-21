use xhummingbird_server::protos::request::Request;
use protobuf::Message;
use protobuf::RepeatedField;
use protobuf::well_known_types::Timestamp;
use std::collections::HashMap;
use std::thread;
use std::time::{Duration, SystemTime};
use std::convert::TryFrom;

fn main(){
    println!("send_request started.");

    let context = zmq::Context::new();
    let publisher = context.socket(zmq::PUB).unwrap();
    assert!(publisher.connect("tcp://localhost:8800").is_ok());

    // Send requests to localhost 100 times
    for n in 0..100 {
        let request = build_sample_request(n);
        println!("{:?}", request);
        let bytes = request.write_to_bytes().unwrap();

        publisher.send(&bytes, 0).unwrap();
        thread::sleep(Duration::from_millis(1000));
    }

    println!("send_request finished.");
}

fn build_sample_request(n: u32) -> Request {
    let mut request = Request::new();
    request.set_level(1);
    request.set_title("UnknownError".to_string());
    request.set_message(format!("Something wrong #{}", n));

    let trace = RepeatedField::from_vec(
        vec!(
            "trace 1".to_string(),
            "trace 2".to_string(),
            "trace 3".to_string(),
        )
    );
    request.set_trace(trace);

    let mut tags = HashMap::new();
    tags.insert("key".to_string(), "value".to_string());
    request.set_tags(tags);

    let mut timestamp = Timestamp::new();
    timestamp.set_seconds(TryFrom::try_from(SystemTime::now().elapsed().unwrap().as_secs()).unwrap());
    request.set_timestamp(timestamp);

    request
}
