use crate::messages::PutEvent;
use crate::actors::storage_actor::StorageActor;
use crate::actors::notification_actor::NotificationActor;
use crate::protos::event::Event;
use std::thread;
use actix::prelude::*;
use protobuf::Message;

pub fn start(storage_actor_address: Addr<StorageActor>, notification_actor_address: Addr<NotificationActor>){
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
