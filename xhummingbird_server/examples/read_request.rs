use xhummingbird_server::protos::request::Request;
use protobuf::Message;
use std::fs;

fn main(){
    println!("read_request started.");

    // Read request from request.bin
    let bytes = fs::read("request.bin").unwrap();
    let request = Request::parse_from_bytes(&bytes).unwrap();
    println!("{:?}", request);

    println!("read_request finished.");
}
