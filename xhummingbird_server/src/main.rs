use xhummingbird_server::protos::request::Request;
use protobuf::Message;

fn main() {
    let address = "tcp://*:8800";
    let context = zmq::Context::new();
    let subscriber = context.socket(zmq::SUB).unwrap();

    assert!(subscriber.bind(address).is_ok());
    assert!(subscriber.set_subscribe("".as_bytes()).is_ok()); // NOTE: Subscribe all

    println!("xHummingbird server started at {}", address);

    loop {
        let bytes = subscriber.recv_bytes(0).unwrap();
        let request = Request::parse_from_bytes(&bytes).unwrap();
        println!("{:?}", request);
    }
}
