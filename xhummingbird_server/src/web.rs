use crate::actors::storage_actor::StorageActor;
use crate::helper;
use crate::messages::*;
use crate::protos::event::Event;

use std::collections::HashMap;

use actix::prelude::*;
use actix_web::{get, web, App, HttpResponse, HttpServer, Responder};
use chrono::{TimeZone, Utc};
use sailfish::TemplateOnce;
use serde::Deserialize;
use serde_json::json;

pub fn start(storage_actor_address: Addr<StorageActor>) {
    let address = "0.0.0.0:8801";

    HttpServer::new(move || {
        App::new()
            .data(WebState {
                storage_actor: storage_actor_address.clone(),
            })
            .service(root)
            .service(events_root)
            .service(event_item)
            .service(config)
            .service(actix_files::Files::new("/static", "./static"))
    })
    .bind(address)
    .unwrap()
    .disable_signals()
    .run();

    info!("xHummingbird web server started at {}", address);
}

#[derive(TemplateOnce)]
#[template(path = "index.html")]
struct RootTemplate {}

struct DisplayableEvent {
    id: u64,
    level: u32,
    title: String,
    message: String,
    trace: Vec<String>,
    tags: Vec<(String, String)>,
    timestamp_rfc2822: String,
    service: String,
}

#[derive(TemplateOnce)]
#[template(path = "events.html")]
struct EventsTemplate {
    events: Vec<DisplayableEvent>,
    next_link: Option<String>,
    from: Option<u64>,
    title: Option<String>,
    stat_array: String,
    titles: Option<Vec<EventSummary>>,
}

impl DisplayableEvent {
    pub fn from_event(event: &Event) -> DisplayableEvent {
        let id = helper::timestamp_u64(event);
        let timestamp = event.get_timestamp();
        let utc = Utc.timestamp(timestamp.get_seconds(), 0);
        let timestamp_rfc2822 = utc.to_rfc2822();

        DisplayableEvent {
            id,
            level: event.get_level(),
            title: event.get_title().to_string(),
            message: event.get_message().to_string(),
            trace: event.get_trace().to_vec(),
            tags: event
                .get_tags()
                .iter()
                .map(|(k, v)| (k.clone(), v.clone()))
                .collect(),
            timestamp_rfc2822,
            service: event.get_service().to_string(),
        }
    }

    pub fn events_link(&self) -> String {
        let mut encoded = form_urlencoded::Serializer::new(String::new());
        encoded.append_pair("title", &self.title);

        format!("/events?{}", encoded.finish())
    }

    pub fn event_link(&self) -> String {
        format!("/events/{}", self.id)
    }
}

struct WebState {
    storage_actor: Addr<StorageActor>,
}

#[get("/")]
async fn root() -> impl Responder {
    let tmpl = RootTemplate {};
    let body = tmpl.render_once().unwrap();
    HttpResponse::Ok().content_type("text/html").body(body)
}

#[get("/events")]
async fn events_root(info: web::Query<EventsInfo>, data: web::Data<WebState>) -> impl Responder {
    let storage_actor = &data.storage_actor;
    let events: Vec<Event> = storage_actor
        .send(HeadEvents {
            from: info.from,
            title: info.title.clone(),
        })
        .await
        .unwrap()
        .unwrap();
    let displayable_events = events
        .iter()
        .map(|event| DisplayableEvent::from_event(event))
        .collect();

    let mut next_link: Option<String> = None;

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

    let stat: Vec<u64> = storage_actor
        .send(StatEvents {
            title: info.title.clone(),
        })
        .await
        .unwrap()
        .unwrap();
    let json_stat = json!(stat);
    let stat_array = json_stat.to_string();

    let titles = match info.title {
        Some(_) => None,
        None => Some(storage_actor.send(GetTitles {}).await.unwrap().unwrap()),
    };

    let tmpl = EventsTemplate {
        events: displayable_events,
        next_link,
        from: info.from,
        title: info.title.clone(),
        stat_array,
        titles,
    };
    let body = tmpl.render_once().unwrap();
    HttpResponse::Ok().content_type("text/html").body(body)
}

#[derive(Deserialize)]
struct EventsInfo {
    from: Option<u64>,
    title: Option<String>,
}

#[get("/events/{id}")]
async fn event_item(web::Path(id): web::Path<u64>, data: web::Data<WebState>) -> impl Responder {
    let storage_actor = &data.storage_actor;
    let result = storage_actor.send(GetEvent { id }).await.unwrap();

    match result {
        Ok(event) => {
            let displayable_event = DisplayableEvent::from_event(&event);

            let tmpl = EventTemplate {
                id: id,
                event: displayable_event,
            };

            let body = tmpl.render_once().unwrap();
            HttpResponse::Ok().content_type("text/html").body(body)
        }
        Err(_) => {
            let tmpl = NotFoundTemplate {
                message: format!("Unknown event id:{}", id),
            };
            let body = tmpl.render_once().unwrap();
            HttpResponse::NotFound()
                .content_type("text/html")
                .body(body)
        }
    }
}

#[derive(TemplateOnce)]
#[template(path = "event.html")]
struct EventTemplate {
    id: u64,
    event: DisplayableEvent,
}

#[derive(TemplateOnce)]
#[template(path = "not_found.html")]
struct NotFoundTemplate {
    message: String,
}

#[derive(TemplateOnce)]
#[template(path = "config.html")]
struct ConfigTemplate {}

#[get("/config")]
async fn config() -> impl Responder {
    let tmpl = ConfigTemplate {};
    let body = tmpl.render_once().unwrap();
    HttpResponse::Ok().content_type("text/html").body(body)
}
