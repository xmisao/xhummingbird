use xhummingbird_server::protos::request::Request;
use protobuf::Message;
use std::thread;
use std::sync::mpsc;

fn main() {
    let address = "tcp://*:8800";
    let context = zmq::Context::new();
    let subscriber = context.socket(zmq::SUB).unwrap();
    let mut store = Store::new();

    assert!(subscriber.bind(address).is_ok());
    assert!(subscriber.set_subscribe("".as_bytes()).is_ok()); // NOTE: Subscribe all

    println!("xHummingbird server started at {}", address);

    loop {
        let bytes = subscriber.recv_bytes(0).unwrap();
        let request = Request::parse_from_bytes(&bytes).unwrap();

        // println!("{:?}", request);
        store.put(request);
        store.print();
    }
}

struct Store {
    data: Vec<Request>
}

impl Store {
    pub fn new() -> Store{
        Store{
            data: Vec::new()
        }
    }

    pub fn put(&mut self, request: Request){
        self.data.push(request);
    }

    pub fn print(&self){
        for event in &self.data {
            println!("{:?}", event);
        }
    }
}
