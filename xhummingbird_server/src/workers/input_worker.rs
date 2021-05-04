use crate::actors::control_actor::ControlActor;
use crate::messages::CommandInput;
use actix::prelude::*;
use std::io;
use std::thread;

pub fn start(control_actor_address: Addr<ControlActor>) {
    thread::spawn(move || loop {
        let mut input = String::new();

        match io::stdin().read_line(&mut input) {
            Ok(_) => {
                let command = input.trim().to_string();
                control_actor_address
                    .try_send(CommandInput { command })
                    .unwrap();
            }
            Err(error) => error!("Error: {}", error),
        }
    });
}
