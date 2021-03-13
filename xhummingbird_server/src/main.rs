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
use actix_web::{get, web, App, HttpServer, HttpResponse, Responder};
use sailfish::TemplateOnce;
use chrono::{Utc, TimeZone};

fn main() {
    ctrlc::set_handler(move || {
        std::process::exit(0);
    }).unwrap();

    let slack_incoming_webhook_endpoint:&str = &env::var("XH_SLACK_INCOMING_WEBHOOK_ENDPOINT").unwrap();
    let slack = Slack::new(slack_incoming_webhook_endpoint).unwrap();

    let store = Arc::new(Mutex::new(Store::new()));
    let storage_reference = Arc::clone(&store);
    let control_reference = Arc::clone(&store);
    let web_server_reference = Arc::clone(&store);

    let (tx1, rx1) = channel();
    let (tx2, rx2) = channel();

    let receiver_thread = start_receiver_thread(tx1, tx2);
    let storage_thread = start_storage_thread(rx1, storage_reference);
    let notification_thread = start_notification_thread(rx2, slack);
    let control_thread = start_control_thread(control_reference);
    let web_server_thread = start_web_server_thread(web_server_reference);

    receiver_thread.join().unwrap();
    storage_thread.join().unwrap();
    control_thread.join().unwrap();
    notification_thread.join().unwrap();
    web_server_thread.join().unwrap();
}

#[derive(TemplateOnce)]
#[template(path = "index.html")]
struct RootTemplate {
}

struct DisplayableEvent {
    level: u32,
    title: String,
    message: String,
    trace: Vec<String>,
    tags: Vec<(String, String)>,
    timestamp_rfc2822: String,

}

#[derive(TemplateOnce)]
#[template(path = "events.html")]
struct EventsTemplate {
    events: Vec<DisplayableEvent>
}


impl DisplayableEvent {
    pub fn from_event(event: &Event) -> DisplayableEvent{
        let timestamp = event.get_timestamp();
        let utc = Utc.timestamp(timestamp.get_seconds(), 0);
        let timestamp_rfc2822 = utc.to_rfc2822();

        DisplayableEvent{
            level: event.get_level(),
            title: event.get_title().to_string(),
            message: event.get_message().to_string(),
            trace: event.get_trace().to_vec(),
            tags: event.get_tags().iter().map(|(k, v)| (k.clone(), v.clone())).collect(),
            timestamp_rfc2822,
        }
    }
}

struct WebState {
    store_reference: Arc<Mutex<Store>>
}

fn start_web_server_thread(store_reference: Arc<Mutex<Store>>) -> JoinHandle<()> {
    thread::spawn(move || {
        start_web_server(store_reference).unwrap();
    })
}

#[get("/")]
async fn root() -> impl Responder {
    let tmpl = RootTemplate{};
    let body = tmpl.render_once().unwrap();
    HttpResponse::Ok().content_type("text/html").body(body)
}

#[get("/events")]
async fn events_root(data: web::Data<WebState>) -> impl Responder {
    let store_reference = &data.store_reference;
    let store = store_reference.lock().unwrap();
    let events = store.head().iter().map(|event| DisplayableEvent::from_event(event)).collect();
    let tmpl = EventsTemplate{events};
    let body = tmpl.render_once().unwrap();
    HttpResponse::Ok().content_type("text/html").body(body)
}

#[actix_web::main]
async fn start_web_server(store_reference: Arc<Mutex<Store>>) -> std::io::Result<()> {
    let address = "0.0.0.0:8801";

    println!("xHummingbird web server started at {}", address);

    HttpServer::new(move ||
        App::new()
            .data(WebState{store_reference: store_reference.clone()})
            .service(root)
            .service(events_root)
        ).bind(address)?
         .run()
         .await
}

fn start_receiver_thread(tx1: Sender<Event>, tx2: Sender<Event>) -> JoinHandle<Thread> {
    thread::spawn(move || {
        let address = "tcp://*:8800";
        let context = zmq::Context::new();
        let subscriber = context.socket(zmq::PULL).unwrap();
        assert!(subscriber.bind(address).is_ok());

        println!("xHummingbird event receiver started at {}", address);

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
                            for event in store.head() {
                                println!("{:?}", event);
                            }
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

    pub fn head(&self) -> Vec<&Event>{
        self.data.iter().rev().take(10).map(|t| t.1).collect()
    }
}
