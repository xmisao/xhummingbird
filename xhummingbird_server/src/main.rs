use xhummingbird_server::protos::event::Event;
use protobuf::Message;
use std::sync::{Mutex, Arc};
use std::thread;
use std::thread::Thread;
use std::thread::JoinHandle;
use std::io;
use std::collections::BTreeMap;
use std::convert::TryInto;
use std::sync::mpsc::{Sender, Receiver};
use std::sync::mpsc::channel;

fn main() {
    let store = Arc::new(Mutex::new(Store::new()));
    let storage_reference = Arc::clone(&store);
    let control_reference = Arc::clone(&store);

    let (tx, rx) = channel();

    let receiver_thread = start_receiver_thread(tx);
    let storage_thread = start_storage_thread(rx, storage_reference);
    let control_thread = start_control_thread(control_reference);

    receiver_thread.join().unwrap();
    storage_thread.join().unwrap();
    control_thread.join().unwrap();
}

fn start_receiver_thread(tx: Sender<Event>) -> JoinHandle<Thread> {
    thread::spawn(move || {
        let address = "tcp://*:8800";
        let context = zmq::Context::new();
        let subscriber = context.socket(zmq::SUB).unwrap();
        assert!(subscriber.bind(address).is_ok());
        assert!(subscriber.set_subscribe("".as_bytes()).is_ok()); // NOTE: Subscribe all

        println!("xHummingbird server started at {}", address);

        loop {
            let bytes = subscriber.recv_bytes(0).unwrap();
            let event = Event::parse_from_bytes(&bytes).unwrap();

            tx.send(event).unwrap();
        }
    })
}

fn start_storage_thread(rx: Receiver<Event>, store_reference: Arc<Mutex<Store>>) -> JoinHandle<Thread> {
    thread::spawn(move || {
        loop {
            let event = rx.recv().unwrap();

            let mut store = store_reference.lock().unwrap();
            store.put(event);
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
                        "head" => {
                            println!("Events:");
                            let store = store_reference.lock().unwrap();
                            store.head();
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
    data: BTreeMap<u64, Event>
}

impl Store {
    pub fn new() -> Store{
        Store{
            data: BTreeMap::new()
        }
    }

    pub fn put(&mut self, event: Event){
        let nsec:u64 = event.get_timestamp().get_nanos().try_into().unwrap();
        let sec:u64 = event.get_timestamp().get_seconds().try_into().unwrap();
        let time = sec * 1_000_000_000 + nsec;

        self.data.insert(time, event);
    }

    pub fn head(&self){
        for (_, event) in self.data.iter().rev().take(10) {
            println!("{:?}", event);
        }
    }
}
