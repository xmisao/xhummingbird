use crate::messages::{PutEvent, HeadEvents, GetEvent, SaveSnapshot, StatEvents};
use crate::protos::event::Event;
use crate::store::Store;
use actix::prelude::*;
use protobuf::Message;
use std::env;

pub struct StorageActor{
    pub store: Store
}

impl Actor for StorageActor{
    type Context = Context<Self>;
}

impl Handler<PutEvent> for StorageActor {
    type Result = std::result::Result<(), ()>;

    fn handle(&mut self, msg: PutEvent, _ctx: &mut Context<Self>) -> Self::Result {
        self.store.put(msg.event);
        Ok(())
    }
}

impl Handler<HeadEvents> for StorageActor {
    type Result = std::result::Result<Vec<Event>, ()>;

    fn handle(&mut self, msg: HeadEvents, _ctx: &mut Context<Self>) -> Self::Result {
        let mut events = Vec::new();

        for event in self.store.head(msg.from, msg.title) {
            events.push(event.clone());
        }

        Ok(events)
    }
}

impl Handler<GetEvent> for StorageActor {
    type Result = std::result::Result<Event, ()>;

    fn handle(&mut self, msg: GetEvent, _ctx: &mut Context<Self>) -> Self::Result {
        match self.store.get(msg.id) {
            Some(event) => Ok(event.clone()),
            None => Err(())
        }
    }
}

impl Handler<SaveSnapshot> for StorageActor {
    type Result = std::result::Result<usize, std::io::Error>;

    fn handle(&mut self, msg: SaveSnapshot, _ctx: &mut Context<Self>) -> Self::Result {
        let path = &env::var("XH_SNAPSHOT").unwrap();
        let result = self.store.save(path);

        match &result {
            Ok(n) => println!("{} events saved.", n),
            Err(e) => println!("Save failed {:?}", e),
        };
        
        result
    }
}

impl Handler<StatEvents> for StorageActor {
    type Result = std::result::Result<Vec<u64>, ()>;

    fn handle(&mut self, msg: StatEvents, _ctx: &mut Context<Self>) -> Self::Result {
        let stat = self.store.stat(msg.title);

        Ok(stat)
    }
}
