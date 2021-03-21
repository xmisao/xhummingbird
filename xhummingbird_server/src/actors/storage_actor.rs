use crate::messages::{PutEvent, HeadEvents};
use crate::protos::event::Event;
use crate::store::Store;
use actix::prelude::*;

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
        println!("PutEvent Hundler {:?}", self.store.head(None));
        Ok(())
    }
}

impl Handler<HeadEvents> for StorageActor {
    type Result = std::result::Result<Vec<Event>, ()>;

    fn handle(&mut self, msg: HeadEvents, _ctx: &mut Context<Self>) -> Self::Result {
        let mut events = Vec::new();

        for event in self.store.head(msg.from) {
            events.push(event.clone());
        }

        Ok(events)
    }
}
