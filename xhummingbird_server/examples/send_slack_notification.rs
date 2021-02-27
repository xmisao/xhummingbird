extern crate slack_hook;
use slack_hook::{Slack, PayloadBuilder};
use std::env;

fn main() {
    let slack_incoming_webhook_endpoint:&str = &env::var("XH_SLACK_INCOMING_WEBHOOK_ENDPOINT").unwrap();

    let slack = Slack::new(slack_incoming_webhook_endpoint).unwrap();

    let p = PayloadBuilder::new()
      .text("xHummingbird notification")
      .channel("#test")
      .username("xHummingbird")
      .icon_emoji(":exclamation:")
      .build()
      .unwrap();

    let res = slack.send(&p);
    match res {
        Ok(()) => println!("ok"),
        Err(x) => println!("ERR: {:?}",x)
    }
}
