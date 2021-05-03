use crate::protos::event::Event;
use crate::messages::PutEvent;
use crate::actors::storage_actor::StorageActor;
use crate::config;
use actix::prelude::*;
use protobuf::Message;
use std::fs::File;
use std::io::{self, Read, BufReader};
use std::convert::TryFrom;
use std::thread;
use std::path::Path;

pub fn start(storage_actor_address: Addr<StorageActor>) {
    thread::spawn(move || {
        let path = &config::snapshot();
        println!("Loaded {} events", load_from_file(path, storage_actor_address).unwrap());
    });
}

fn load_from_file(path:&str, storage_actor_address: Addr<StorageActor>) -> Result<usize, io::Error> {
    if Path::new(path).exists() {
        let mut size_buf = [0; 4]; // NOTE: u32
        let mut reader = BufReader::new(File::open(path)?);
        let mut n = 0;

        loop {
            reader.read_exact(&mut size_buf)?;

            let size:usize = TryFrom::try_from(u32::from_ne_bytes(size_buf)).unwrap();

            if size == 0 {
                break;
            }

            let mut event_buf = vec![0; size];
            reader.read_exact(&mut event_buf)?;

            let event = Event::parse_from_bytes(&event_buf)?;
            storage_actor_address.try_send(PutEvent{event: event.clone()}).ok();

            n += 1;
        }

        Ok(n)
    } else {
        println!("{} does not exist.", path);
        Ok(0)
    }
}
