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
use actix::prelude::*;

// #[actix_web::main]
// async fn main() {
fn main() {
    ctrlc::set_handler(move || {
        std::process::exit(0);
    }).unwrap();

    let mut sys = actix::System::new("app");

    let slack_incoming_webhook_endpoint:&str = &env::var("XH_SLACK_INCOMING_WEBHOOK_ENDPOINT").unwrap();
    let slack = Slack::new(slack_incoming_webhook_endpoint).unwrap();

    let store = Store::new();
    let storage_actor = StorageActor{store};
    let storage_actor_address = storage_actor.start();

    let control_reference = storage_actor_address.clone();
    let web_server_reference = storage_actor_address.clone();

    let (tx2, rx2) = channel();

    let receiver_thread = start_receiver_thread(storage_actor_address.clone(), tx2);
    let notification_thread = start_notification_thread(rx2, slack);
    let control_thread = start_control_thread(control_reference);

    // let srv = start_web_server(web_server_reference);
    let address = "0.0.0.0:8801";

    let srv = HttpServer::new(move ||
                              App::new()
                              .data(WebState{storage_actor: storage_actor_address.clone()})
                              .service(root)
                              .service(events_root)
                             ).bind(address).unwrap().run();

    println!("xHummingbird web server started at {}", address);

    sys.block_on(srv);

    notification_thread.join().unwrap();
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
    storage_actor: Addr<StorageActor>
}

#[get("/")]
async fn root() -> impl Responder {
    let tmpl = RootTemplate{};
    let body = tmpl.render_once().unwrap();
    HttpResponse::Ok().content_type("text/html").body(body)
}

#[get("/events")]
async fn events_root(data: web::Data<WebState>) -> impl Responder {
    let storage_actor = &data.storage_actor;
    let events:Vec<Event> = storage_actor.send(HeadEvents{}).await.unwrap().unwrap();
    let events = events.iter().map(|event| DisplayableEvent::from_event(event)).collect();
    let tmpl = EventsTemplate{events};
    let body = tmpl.render_once().unwrap();
    HttpResponse::Ok().content_type("text/html").body(body)
}

/*
fn start_web_server(storage_actor: Addr<StorageActor>) {
    let address = "0.0.0.0:8801";

    println!("xHummingbird web server started at {}", address);

    HttpServer::new(move ||
        App::new()
            .data(WebState{storage_actor: storage_actor.clone()})
            .service(root)
            .service(events_root)
        ).bind(address)?
         .run()
}
*/

fn start_receiver_thread(storage_actor_address: Addr<StorageActor>, tx2: Sender<Event>){
    actix::spawn(async move {
        let address = "tcp://*:8800";
        let context = zmq::Context::new();
        let subscriber = context.socket(zmq::PULL).unwrap();
        assert!(subscriber.bind(address).is_ok());

        println!("xHummingbird event receiver started at {}", address);

        loop {
            let bytes = subscriber.recv_bytes(0).unwrap();
            let event = Event::parse_from_bytes(&bytes).unwrap();

            storage_actor_address.send(PutEvent{event: event.clone()}).await.unwrap();
            // tx1.send(event.clone()).unwrap();
            tx2.send(event.clone()).unwrap();
        }
    })
}

/*
fn start_storage_thread(rx: Receiver<Event>, store_reference: Arc<Mutex<Store>>) -> JoinHandle<Thread> {
    thread::spawn(move || {
        loop {
            let event = rx.recv().unwrap();

            let mut store = store_reference.lock().unwrap();
            store.put(event);
        }
    })
}
*/

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

fn start_control_thread(storage_actor: Addr<StorageActor>){
    actix::spawn(async move {
        loop {
            let mut input = String::new();

            match io::stdin().read_line(&mut input) {
                Ok(_) => {
                    let input = input.trim();

                    match &*input {
                        "head" => {
                            println!("Events:");
                            let s1 = storage_actor.send(HeadEvents{});
                            println!("s1 done");
                            let s2 = s1.await;
                            println!("s2: {:?}", s2);
                            let s3 = s2.unwrap();
                            println!("s3: {:?}", s3);
                            let s4 = s3.unwrap();
                            println!("s4: {:?}", s4);

                            // for event in storage_actor.send(HeadEvents{}).await.unwrap().unwrap() {
                            for event in s4 {
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

struct StorageActor{
    store: Store
}

impl Actor for StorageActor{
    type Context = Context<Self>;
}

impl Handler<PutEvent> for StorageActor {
    type Result = std::result::Result<(), ()>;

    fn handle(&mut self, msg: PutEvent, _ctx: &mut Context<Self>) -> Self::Result {
        self.store.put(msg.event);
        Ok(())
    }
}

#[derive(Message)]
#[rtype(result = "std::result::Result<(), ()>")]
struct PutEvent{
    event: Event
}

impl Handler<HeadEvents> for StorageActor {
    type Result = std::result::Result<Vec<Event>, ()>;

    fn handle(&mut self, _msg: HeadEvents, _ctx: &mut Context<Self>) -> Self::Result {
        let mut events = Vec::new();

        for event in self.store.head() {
            events.push(event.clone());
        }

        Ok(events)
    }
}

#[derive(Message)]
#[rtype(result = "std::result::Result<(Vec<Event>), ()>")]
struct HeadEvents{
}
