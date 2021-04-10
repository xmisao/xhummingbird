use crate::messages::PutEvent;
use actix::prelude::*;
use std::collections::HashMap;
use std::time::Duration;

pub struct NotificationActor{
    pub slack_incoming_webhook_endpoint: String
}

impl Actor for NotificationActor{
    type Context = Context<Self>;
}

impl Handler<PutEvent> for NotificationActor {
    type Result = std::result::Result<(), ()>;

    fn handle(&mut self, msg: PutEvent, _ctx: &mut Context<Self>) -> Self::Result {
        let event = msg.event;

        let text = format!("title: {}\nmessage: {}", event.get_title(), event.get_message());

        let mut params = HashMap::new();
        params.insert("text", text);

        let client = reqwest::blocking::Client::builder().timeout(Duration::from_secs(5)).build().unwrap();

        let res = client.post(&self.slack_incoming_webhook_endpoint).json(&params).send();

        match res {
            Ok((_)) => (),
            Err(x) => println!("Notification error: {:?}", x)
        }

        Ok(())
    }
}

