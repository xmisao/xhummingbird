use std::collections::BTreeMap;
use std::convert::TryInto;
use std::env;
use std::io;
use std::thread::{Thread, JoinHandle};
use std::thread;
use std::time;
use xhummingbird_server::protos::event::Event;
use protobuf::Message;
extern crate slack_hook;
use slack_hook::{Slack, PayloadBuilder};
use actix_web::{get, web, App, HttpServer, HttpResponse, Responder};
use sailfish::TemplateOnce;
use chrono::{Utc, TimeZone};
use actix::prelude::*;

fn main() {
    ctrlc::set_handler(move || {
        std::process::exit(0);
    }).unwrap();

    let mut sys = actix::System::new("app");

    let slack_incoming_webhook_endpoint:&str = &env::var("XH_SLACK_INCOMING_WEBHOOK_ENDPOINT").unwrap();
    let slack = Slack::new(slack_incoming_webhook_endpoint).unwrap();
    let notification_actor = NotificationActor{slack};
    let notification_actor_address= notification_actor.start();

    let store = Store::new();
    let storage_actor = StorageActor{store};
    let storage_actor_address = storage_actor.start();

    let web_server_reference = storage_actor_address.clone();

    let receiver_thread = start_receiver_thread(storage_actor_address.clone(), notification_actor_address.clone());

    let addr = storage_actor_address.clone();

    let control_actor = ControlActor{storage_actor_address: storage_actor_address.clone()};
    let control_actor_address = control_actor.start();
    start_input_thread(control_actor_address);

    let address = "0.0.0.0:8801";

    let srv = HttpServer::new(move ||
                              App::new()
                              .data(WebState{storage_actor: storage_actor_address.clone()})
                              .service(root)
                              .service(events_root)
                             ).bind(address).unwrap().run();

    println!("xHummingbird web server started at {}", address);

    sys.block_on(srv);
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

fn start_receiver_thread(storage_actor_address: Addr<StorageActor>, notification_actor_address: Addr<NotificationActor>){
    thread::spawn(move || {
        let address = "tcp://*:8800";
        let context = zmq::Context::new();
        let subscriber = context.socket(zmq::PULL).unwrap();
        assert!(subscriber.bind(address).is_ok());

        println!("xHummingbird event receiver started at {}", address);

        loop {
            let bytes = subscriber.recv_bytes(0).unwrap();
            let event = Event::parse_from_bytes(&bytes).unwrap();
            println!("{:?}", event);

            let storage_actor_address = storage_actor_address.clone();

            println!("{:?}", storage_actor_address.try_send(PutEvent{event: event.clone()}).unwrap());
            println!("{:?}", notification_actor_address.try_send(PutEvent{event: event.clone()}).unwrap());
        }
    });
}

fn start_input_thread(control_actor_address: Addr<ControlActor>){
    thread::spawn(move ||{
        loop {
            let mut input = String::new();

            match io::stdin().read_line(&mut input) {
                Ok(_) => {
                    let command = input.trim().to_string();
                    control_actor_address.try_send(CommandInput{command});
                },
                Err(error) => println!("Error: {}", error),
            }
        }
    });
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
        println!("PutEvent Hundler {:?}", self.store.head());
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

struct NotificationActor{
    slack: Slack
}

impl Actor for NotificationActor{
    type Context = Context<Self>;
}

impl Handler<PutEvent> for NotificationActor {
    type Result = std::result::Result<(), ()>;

    fn handle(&mut self, msg: PutEvent, _ctx: &mut Context<Self>) -> Self::Result {
        let event = msg.event;

        let p = PayloadBuilder::new()
            .text(format!("title: {}\nmessage: {}", event.get_title(), event.get_message()))
            .username("xHummingbird")
            .icon_emoji(":exclamation:")
            .build()
            .unwrap();

        let res = self.slack.send(&p);

        match res {
            Ok(()) => (),
            Err(x) => println!("Notification error: {:?}", x)
        }

        Ok(())
    }
}

struct ControlActor{
    storage_actor_address: Addr<StorageActor>
}

impl Actor for ControlActor{
    type Context = Context<Self>;
}

#[derive(Message)]
#[rtype(result = "std::result::Result<(), ()>")]
struct CommandInput{
    command: String
}

impl Handler<CommandInput> for ControlActor {
    type Result = std::result::Result<(), ()>;

    fn handle(&mut self, msg: CommandInput, _ctx: &mut Context<Self>) -> Self::Result {
        let command = msg.command;
        let storage_actor_address= self.storage_actor_address.clone();

        actix::spawn(async move {
            match &*command {
                "head" => {
                    println!("Events:");
                    let s1 = storage_actor_address.send(HeadEvents{}).await.unwrap();
                    println!("s1: {:?}", s1);

                    for event in s1 {
                        println!("{:?}", event);
                    }
                },
                _ => {
                    println!("Unknown command: {}", command);
                }
            }
        });

        Ok(())
    }
}
