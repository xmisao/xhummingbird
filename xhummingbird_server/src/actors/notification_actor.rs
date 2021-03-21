use crate::messages::PutEvent;
use actix::prelude::*;
use slack_hook::{Slack, PayloadBuilder};

pub struct NotificationActor{
    pub slack: Slack
}

impl Actor for NotificationActor{
    type Context = Context<Self>;
}

impl Handler<PutEvent> for NotificationActor {
    type Result = std::result::Result<(), ()>;

    fn handle(&mut self, msg: PutEvent, _ctx: &mut Context<Self>) -> Self::Result {
        let event = msg.event;

        let p = PayloadBuilder::new()
            .text(format!("title: {}\nmessage: {}", event.get_title(), event.get_message()))
            .username("xHummingbird")
            .icon_emoji(":exclamation:")
            .build()
            .unwrap();

        let res = self.slack.send(&p);

        match res {
            Ok(()) => (),
            Err(x) => println!("Notification error: {:?}", x)
        }

        Ok(())
    }
}

