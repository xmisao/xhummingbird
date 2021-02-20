use xhummingbird_server::protos::request::Request;
use protobuf::Message;
use protobuf::RepeatedField;
use std::collections::HashMap;
use std::fs::File;
use std::io::prelude::*;

fn main(){
    println!("write_request started.");

    let mut request = Request::new();
    request.set_level(1);
    request.set_title("UnknownError".to_string());
    request.set_message("Something wrong".to_string());

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

    println!("{:?}", request);

    // Write request to request.bin
    let mut file = File::create("request.bin").unwrap();
    let bytes = request.write_to_bytes().unwrap();
    file.write(&bytes);

    println!("write_request finished.");
}
