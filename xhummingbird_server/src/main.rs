use xhummingbird_server::workers::*;
use xhummingbird_server::actors::storage_actor::StorageActor;
use xhummingbird_server::actors::control_actor::ControlActor;
use xhummingbird_server::actors::notification_actor::NotificationActor;
use xhummingbird_server::store::Store;
use xhummingbird_server::web;

use std::env;

use actix::prelude::*;
extern crate slack_hook;
use slack_hook::Slack;

fn main() {
    ctrlc::set_handler(move || {
        std::process::exit(0);
    }).unwrap();

    let sys = actix::System::new("app");

    let slack_incoming_webhook_endpoint:&str = &env::var("XH_SLACK_INCOMING_WEBHOOK_ENDPOINT").unwrap();
    let slack = Slack::new(slack_incoming_webhook_endpoint).unwrap();
    let notification_actor = NotificationActor{slack};
    let notification_actor_address= notification_actor.start();

    let store = Store::new();
    let storage_actor = StorageActor{store};
    let storage_actor_address = storage_actor.start();

    receiver_worker::start(storage_actor_address.clone(), notification_actor_address.clone());

    let control_actor = ControlActor{storage_actor_address: storage_actor_address.clone()};
    let control_actor_address = control_actor.start();
    input_worker::start(control_actor_address);

    web::start(storage_actor_address.clone());

    let _ = sys.run();
}
