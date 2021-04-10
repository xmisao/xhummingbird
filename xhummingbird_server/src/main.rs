use xhummingbird_server::workers::*;
use xhummingbird_server::actors::storage_actor::StorageActor;
use xhummingbird_server::actors::control_actor::ControlActor;
use xhummingbird_server::actors::notification_actor::NotificationActor;
use xhummingbird_server::store::Store;
use xhummingbird_server::web;
use xhummingbird_server::messages::{SaveSnapshot};
use xhummingbird_server::loader;

use std::env;
use std::time::Duration;
use std::thread;

use actix::prelude::*;

fn main() {
    let sys = actix::System::new("app");

    let slack_incoming_webhook_endpoint = env::var("XH_SLACK_INCOMING_WEBHOOK_ENDPOINT").unwrap().to_string();
    let default_notification_threshold = "0".to_string();
    let notification_threshold = env::var("XH_NOTIFICATION_THRESHOLD").unwrap_or(default_notification_threshold).parse::<u32>().unwrap();

    println!("Notify Slack when receiving an event that has a level greater equal than {}", notification_threshold);

    let notification_actor = NotificationActor{slack_incoming_webhook_endpoint, notification_threshold};
    let notification_actor_address= notification_actor.start();

    let store = Store::new();
    let storage_actor = StorageActor{store};
    let storage_actor_address = storage_actor.start();

    receiver_worker::start(storage_actor_address.clone(), notification_actor_address.clone());

    let control_actor = ControlActor{storage_actor_address: storage_actor_address.clone()};
    let control_actor_address = control_actor.start();
    input_worker::start(control_actor_address);

    web::start(storage_actor_address.clone());

    loader::start(storage_actor_address.clone());

    let storage_actor_address_for_autosave = storage_actor_address.clone();
    actix_rt::spawn(async move {
        let mut interval = actix_rt::time::interval(Duration::from_secs(600));
        interval.tick().await;

        loop {
            interval.tick().await;
            println!("Auto saving...");
            storage_actor_address_for_autosave.try_send(SaveSnapshot{}).ok();
        }
    });

    ctrlc::set_handler(move || {
        storage_actor_address.try_send(SaveSnapshot{}).ok();
        thread::sleep(Duration::from_secs(3)); // FIXME
        std::process::exit(0);
    }).unwrap();

    let _ = sys.run();
}
