use crate::actors::storage_actor::StorageActor;
use crate::protos::event::Event;
use crate::messages::HeadEvents;
use crate::helper;

use actix::prelude::*;
use actix_web::{get, web, App, HttpServer, HttpResponse, Responder};
use sailfish::TemplateOnce;
use chrono::{Utc, TimeZone};
use serde::{Deserialize};

pub fn start(storage_actor_address: Addr<StorageActor>){
    let address = "0.0.0.0:8801";

    HttpServer::new(move ||
                    App::new()
                    .data(WebState{storage_actor: storage_actor_address.clone()})
                    .service(root)
                    .service(events_root)
                    .service(actix_files::Files::new("/static", "./static"))
                   ).bind(address).unwrap().run();

    println!("xHummingbird web server started at {}", address);
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
    events: Vec<DisplayableEvent>,
    next_link: Option<String>,
    from: Option<u64>,
    title: Option<String>,
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

    pub fn events_link(&self) -> String{
        let mut encoded = form_urlencoded::Serializer::new(String::new());
        encoded.append_pair("title", &self.title);

        format!("/events?{}", encoded.finish())
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
async fn events_root(info: web::Query<EventsInfo>, data: web::Data<WebState>) -> impl Responder {
    let storage_actor = &data.storage_actor;
    let events:Vec<Event> = storage_actor.send(HeadEvents{from: info.from, title: info.title.clone()}).await.unwrap().unwrap();
    let displayable_events = events.iter().map(|event| DisplayableEvent::from_event(event)).collect();

    let mut next_link:Option<String> = None;

    let last_event = events.last();

    if last_event != None {
        let mut encoded = form_urlencoded::Serializer::new(String::new());

        let next_from = helper::timestamp_u64(last_event.unwrap());
        encoded.append_pair("from", &format!("{}", next_from));

        if info.title != None {
            encoded.append_pair("title", &info.title.clone().unwrap());
        }

        next_link = Some(format!("/events?{}", encoded.finish()));
    }

    let tmpl = EventsTemplate{events: displayable_events, next_link, from: info.from, title: info.title.clone()};
    let body = tmpl.render_once().unwrap();
    HttpResponse::Ok().content_type("text/html").body(body)
}

#[derive(Deserialize)]
struct EventsInfo {
    from: Option<u64>,
    title: Option<String>,
}
