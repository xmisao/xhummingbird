use crate::messages::{CommandInput, HeadEvents, SaveSnapshot};
use crate::actors::storage_actor::StorageActor;
use actix::prelude::*;
use crate::loader;

pub struct ControlActor{
    pub storage_actor_address: Addr<StorageActor>
}

impl Actor for ControlActor{
    type Context = Context<Self>;
}

impl Handler<CommandInput> for ControlActor {
    type Result = std::result::Result<(), ()>;

    fn handle(&mut self, msg: CommandInput, _ctx: &mut Context<Self>) -> Self::Result {
        let command = msg.command;
        let storage_actor_address= self.storage_actor_address.clone();

        actix::spawn(async move {
            match &*command {
                "head" => {
                    println!("Events:");
                    let s1 = storage_actor_address.send(HeadEvents{from: None, title: None}).await.unwrap();
                    println!("s1: {:?}", s1);

                    for event in s1 {
                        println!("{:?}", event);
                    }
                },
                "load" => {
                    loader::start(storage_actor_address.clone());
                },
                "save" => {
                    storage_actor_address.try_send(SaveSnapshot{}).ok();
                }
                _ => {
                    println!("Unknown command: {}", command);
                }
            }
        });

        Ok(())
    }
}
