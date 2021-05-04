#[macro_use]
extern crate log;

use xhummingbird_server::actors::control_actor::ControlActor;
use xhummingbird_server::actors::notification_actor::NotificationActor;
use xhummingbird_server::actors::storage_actor::StorageActor;
use xhummingbird_server::config;
use xhummingbird_server::loader;
use xhummingbird_server::messages::*;
use xhummingbird_server::store::Store;
use xhummingbird_server::web;
use xhummingbird_server::workers::*;

use std::time::Duration;

use actix::prelude::*;

fn main() {
    env_logger::init();

    let sys = actix::System::new("app");

    let slack_incoming_webhook_endpoint = config::slack_incoming_webhook_endpoint();
    let notification_threshold = config::notification_threshold();
    info!(
        "Notify Slack when receiving an event that has a level greater equal than {}",
        notification_threshold
    );

    let mut notification_arbiter = Arbiter::new();
    let notification_actor = NotificationActor {
        slack_incoming_webhook_endpoint,
        notification_threshold,
    };
    let notification_actor_address =
        NotificationActor::start_in_arbiter(&notification_arbiter, |_| notification_actor);

    let mut storage_arbiter = Arbiter::new();
    let store = Store::new();
    let storage_actor = StorageActor { store };
    let storage_actor_address = StorageActor::start_in_arbiter(&storage_arbiter, |_| storage_actor);

    receiver_worker::start(
        storage_actor_address.clone(),
        notification_actor_address.clone(),
    );

    let mut control_arbiter = Arbiter::new();
    let control_actor_address: Option<Addr<ControlActor>> = None;
    if !config::no_control() {
        let control_actor = ControlActor {
            storage_actor_address: storage_actor_address.clone(),
        };
        let control_actor_address = Some(ControlActor::start_in_arbiter(&control_arbiter, |_| {
            control_actor
        }));
        input_worker::start(control_actor_address.unwrap());
    }

    web::start(storage_actor_address.clone());

    loader::start(storage_actor_address.clone());

    let storage_actor_address_for_autosave = storage_actor_address.clone();
    actix_rt::spawn(async move {
        let mut interval = actix_rt::time::interval(Duration::from_secs(600));
        interval.tick().await;

        loop {
            interval.tick().await;
            info!("Auto saving...");
            storage_actor_address_for_autosave
                .try_send(SaveSnapshot {})
                .ok();
        }
    });

    ctrlc::set_handler(move || {
        info!("Start shutdown.");

        storage_actor_address.try_send(SaveSnapshot {}).unwrap();

        match control_actor_address.clone() {
            Some(control_actor_address) => control_actor_address.try_send(Stop {}).unwrap(),
            None => (),
        };
        notification_actor_address.try_send(Stop {}).unwrap();
        storage_actor_address.try_send(Stop {}).unwrap();
    })
    .unwrap();

    let _ = sys.run();
    info!("sys.run() finished.");

    notification_arbiter.join().unwrap();
    storage_arbiter.join().unwrap();
    if !config::no_control() {
        control_arbiter.join().unwrap();
    }

    info!("Shutdown correctly.");
}
