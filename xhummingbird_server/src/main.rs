use xhummingbird_server::protos::request::Request;
use protobuf::Message;
use std::sync::{Mutex, Arc};
use std::thread;
use std::thread::Thread;
use std::thread::JoinHandle;
use std::io;

fn main() {
    let store = Arc::new(Mutex::new(Store::new()));
    let worker_reference = Arc::clone(&store);
    let control_reference = Arc::clone(&store);

    let worker_thread = start_worker_thread(worker_reference);
    let control_thread = start_control_thread(control_reference);

    worker_thread.join().unwrap();
    control_thread.join().unwrap();
}

fn start_worker_thread(store_reference: Arc<Mutex<Store>>) -> JoinHandle<Thread> {
    thread::spawn(move || {
        let address = "tcp://*:8800";
        let context = zmq::Context::new();
        let subscriber = context.socket(zmq::SUB).unwrap();
        assert!(subscriber.bind(address).is_ok());
        assert!(subscriber.set_subscribe("".as_bytes()).is_ok()); // NOTE: Subscribe all

        println!("xHummingbird server started at {}", address);

        loop {
            let bytes = subscriber.recv_bytes(0).unwrap();
            let request = Request::parse_from_bytes(&bytes).unwrap();

            let mut store = store_reference.lock().unwrap();
            store.put(request);
        }
    })
}

fn start_control_thread(store_reference: Arc<Mutex<Store>>) -> JoinHandle<Thread> {
    thread::spawn(move || {
        loop {
            let mut input = String::new();

            match io::stdin().read_line(&mut input) {
                Ok(_) => {
                    let input = input.trim();

                    match &*input {
                        "print" => {
                            println!("Requests:");
                            let store = store_reference.lock().unwrap();
                            store.print();
                        },
                        _ => {
                            println!("Unknown command: {}", input);
                        }

                    }
                },
                Err(error) => println!("Error: {}", error),
            }
        };
    })
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
