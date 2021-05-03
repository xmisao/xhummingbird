use crate::actors::notification_actor::NotificationActor;
use crate::actors::storage_actor::StorageActor;
use crate::messages::PutEvent;
use crate::protos::event::Event;
use actix::prelude::*;
use protobuf::Message;
use std::thread;
use std::time::Duration;

pub fn start(
    storage_actor_address: Addr<StorageActor>,
    notification_actor_address: Addr<NotificationActor>,
) {
    thread::spawn(move || loop {
        run(
            storage_actor_address.clone(),
            notification_actor_address.clone(),
        );
        println!("Unexpected run() aborted.");
        thread::sleep(Duration::from_millis(1000));
    });
}

pub fn run(
    storage_actor_address: Addr<StorageActor>,
    notification_actor_address: Addr<NotificationActor>,
) -> Option<u8> {
    let address = "tcp://*:8800";
    let context = zmq::Context::new();
    let subscriber = context.socket(zmq::PULL).ok()?;
    assert!(subscriber.bind(address).is_ok());

    println!("xHummingbird event receiver started at {}", address);

    loop {
        let bytes = subscriber.recv_bytes(0).ok()?;
        let event = Event::parse_from_bytes(&bytes).ok()?;
        println!("{:?}", event);

        let storage_actor_address = storage_actor_address.clone();

        println!(
            "{:?}",
            storage_actor_address
                .try_send(PutEvent {
                    event: event.clone()
                })
                .ok()?
        );
        println!(
            "{:?}",
            notification_actor_address
                .try_send(PutEvent {
                    event: event.clone()
                })
                .ok()?
        );
    }
}
