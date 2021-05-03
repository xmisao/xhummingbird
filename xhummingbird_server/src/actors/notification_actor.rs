use crate::messages::*;
use actix::prelude::*;
use std::collections::HashMap;
use std::time::Duration;

pub struct NotificationActor{
    pub slack_incoming_webhook_endpoint: String,
    pub notification_threshold: u32,
}

impl Actor for NotificationActor{
    type Context = Context<Self>;

    fn stopped(&mut self, _ctx: &mut Self::Context){
        println!("NotificationActor stopped.");
    }
}

impl Handler<PutEvent> for NotificationActor {
    type Result = std::result::Result<(), ()>;

    fn handle(&mut self, msg: PutEvent, _ctx: &mut Context<Self>) -> Self::Result {
        let event = msg.event;

        if event.level >= self.notification_threshold {
            let text = format!("title: {}\nmessage: {}", event.get_title(), event.get_message());

            let mut params = HashMap::new();
            params.insert("text", text);

            let client = reqwest::blocking::Client::builder().timeout(Duration::from_secs(5)).build().unwrap();

            let res = client.post(&self.slack_incoming_webhook_endpoint).json(&params).send();

            match res {
                Ok(_) => (),
                Err(x) => println!("Notification error: {:?}", x)
            }
        }

        Ok(())
    }
}

impl Handler<Stop> for NotificationActor {
    type Result = std::result::Result<(), ()>;

    fn handle(&mut self, _msg: Stop, ctx: &mut Context<Self>) -> Self::Result {
        Context::stop(ctx);

        Ok(())
    }
}
