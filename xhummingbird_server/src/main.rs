use std::collections::BTreeMap;
use std::convert::TryInto;
use std::env;
use std::io;
use std::sync::mpsc::{channel, Sender, Receiver};
use std::sync::{Mutex, Arc};
use std::thread::{Thread, JoinHandle};
use std::thread;
use xhummingbird_server::protos::event::Event;
use protobuf::Message;
extern crate slack_hook;
use slack_hook::{Slack, PayloadBuilder};
use actix_web::{get, App, HttpServer, Responder};

fn main() {
    ctrlc::set_handler(move || {
        std::process::exit(0);
    }).unwrap();

    let slack_incoming_webhook_endpoint:&str = &env::var("XH_SLACK_INCOMING_WEBHOOK_ENDPOINT").unwrap();
    let slack = Slack::new(slack_incoming_webhook_endpoint).unwrap();

    let store = Arc::new(Mutex::new(Store::new()));
    let storage_reference = Arc::clone(&store);
    let control_reference = Arc::clone(&store);

    let (tx1, rx1) = channel();
    let (tx2, rx2) = channel();

    let receiver_thread = start_receiver_thread(tx1, tx2);
    let storage_thread = start_storage_thread(rx1, storage_reference);
    let notification_thread = start_notification_thread(rx2, slack);
    let control_thread = start_control_thread(control_reference);
    let web_server_thread = start_web_server_thread();

    receiver_thread.join().unwrap();
    storage_thread.join().unwrap();
    control_thread.join().unwrap();
    notification_thread.join().unwrap();
    web_server_thread.join().unwrap();
}

fn start_web_server_thread() -> JoinHandle<()> {
    thread::spawn(move || {
        start_web_server().unwrap();
    })
}

#[get("/")]
async fn root() -> impl Responder {
    "xHummingbird"
}

#[actix_web::main]
async fn start_web_server() -> std::io::Result<()> {
    HttpServer::new(|| App::new().service(root))
        .bind("0.0.0.0:8080")?
        .run()
        .await
}

fn start_receiver_thread(tx1: Sender<Event>, tx2: Sender<Event>) -> JoinHandle<Thread> {
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

            tx1.send(event.clone()).unwrap();
            tx2.send(event.clone()).unwrap();
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

fn start_notification_thread(rx: Receiver<Event>, slack: Slack) -> JoinHandle<Thread> {
    thread::spawn(move || {
        loop {
            let event = rx.recv().unwrap();

            let p = PayloadBuilder::new()
                .text(format!("title: {}\nmessage: {}", event.get_title(), event.get_message()))
                .username("xHummingbird")
                .icon_emoji(":exclamation:")
                .build()
                .unwrap();

            let res = slack.send(&p);

            match res {
                Ok(()) => (),
                Err(x) => println!("Notification error: {:?}", x)
            }
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
