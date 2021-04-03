use crate::messages::{CommandInput, HeadEvents};
use crate::actors::storage_actor::StorageActor;
use actix::prelude::*;

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
                _ => {
                    println!("Unknown command: {}", command);
                }
            }
        });

        Ok(())
    }
}
