use xhummingbird_server::protos::event::Event;
use xhummingbird_server::messages::*;
use xhummingbird_server::workers::*;
use xhummingbird_server::actors::storage_actor::StorageActor;
use xhummingbird_server::actors::control_actor::ControlActor;
use xhummingbird_server::actors::notification_actor::NotificationActor;
use xhummingbird_server::store::Store;

use std::env;

use actix_web::{get, web, App, HttpServer, HttpResponse, Responder};
use actix::prelude::*;
use sailfish::TemplateOnce;
use chrono::{Utc, TimeZone};
extern crate slack_hook;
use slack_hook::Slack;

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

    let receiver_thread = receiver_worker::start_receiver_thread(storage_actor_address.clone(), notification_actor_address.clone());

    let addr = storage_actor_address.clone();

    let control_actor = ControlActor{storage_actor_address: storage_actor_address.clone()};
    let control_actor_address = control_actor.start();
    input_worker::start_input_thread(control_actor_address);

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
