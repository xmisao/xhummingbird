use crate::actors::storage_actor::StorageActor;
use crate::loader;
use crate::messages::*;
use actix::prelude::*;

pub struct ControlActor {
    pub storage_actor_address: Addr<StorageActor>,
}

impl Actor for ControlActor {
    type Context = Context<Self>;

    fn stopped(&mut self, _ctx: &mut Self::Context) {
        info!("ControlActor stopped.");
    }
}

impl Handler<CommandInput> for ControlActor {
    type Result = std::result::Result<(), ()>;

    fn handle(&mut self, msg: CommandInput, _ctx: &mut Context<Self>) -> Self::Result {
        let command = msg.command;
        let storage_actor_address = self.storage_actor_address.clone();

        actix::spawn(async move {
            match &*command {
                "head" => {
                    println!("Events:");
                    let s1 = storage_actor_address
                        .send(HeadEvents {
                            from: None,
                            title: None,
                            service: None,
                        })
                        .await
                        .unwrap();
                    println!("s1: {:?}", s1);

                    for event in s1 {
                        println!("{:?}", event);
                    }
                }
                "stat" => {
                    println!("Stat:");
                    let stat = storage_actor_address
                        .send(StatEvents {
                            title: None,
                            service: None,
                        })
                        .await
                        .unwrap();
                    println!("{:?}", stat);
                }
                "titles" => {
                    println!("Titles:");
                    let titles = storage_actor_address
                        .send(GetTitles {
                            title: None,
                            service: None,
                        })
                        .await
                        .unwrap();
                    println!("{:?}", titles);
                }
                "load" => {
                    loader::start(storage_actor_address.clone());
                }
                "save" => {
                    storage_actor_address.try_send(SaveSnapshot {}).ok();
                }
                _ => {
                    error!("Unknown command: {}", command);
                }
            }
        });

        Ok(())
    }
}

impl Handler<Stop> for ControlActor {
    type Result = std::result::Result<(), ()>;

    fn handle(&mut self, _msg: Stop, ctx: &mut Context<Self>) -> Self::Result {
        Context::stop(ctx);

        Ok(())
    }
}
