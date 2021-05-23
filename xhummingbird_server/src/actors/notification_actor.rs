use crate::messages::*;
use actix::prelude::*;
use chrono::{DateTime, Utc};
use std::collections::{HashMap, HashSet};
use std::time::Duration;

pub struct NotificationActor {
    pub slack_incoming_webhook_endpoint: String,
    pub notification_threshold: u32,
    counter: HashMap<(String, String), NotificationEntry>,
}

impl NotificationActor {
    pub fn new(
        slack_incoming_webhook_endpoint: String,
        notification_threshold: u32,
    ) -> NotificationActor {
        NotificationActor {
            slack_incoming_webhook_endpoint,
            notification_threshold,
            counter: HashMap::new(),
        }
    }
}

impl Actor for NotificationActor {
    type Context = Context<Self>;

    fn stopped(&mut self, _ctx: &mut Self::Context) {
        info!("NotificationActor stopped.");
    }
}

impl Handler<PutEvent> for NotificationActor {
    type Result = std::result::Result<(), ()>;

    fn handle(&mut self, msg: PutEvent, _ctx: &mut Context<Self>) -> Self::Result {
        let notification_interval = chrono::Duration::seconds(-1 * 60 * 60);
        let notification_count: HashSet<u64> = vec![1, 10, 100, 1000, 10000].into_iter().collect();

        let event = msg.event;

        if event.level >= self.notification_threshold {
            let key = (event.service.clone(), event.title.clone());
            if !self.counter.contains_key(&key)
                || self
                    .counter
                    .get(&key)
                    .unwrap()
                    .notified_at
                    .signed_duration_since(Utc::now())
                    > notification_interval
            {
                self.counter.insert(key.clone(), NotificationEntry::new());
            }

            let entry = self.counter.get_mut(&key).unwrap();
            entry.count += 1;

            if notification_count.contains(&entry.count) {
                trace!("Send notification of {:?}", key);

                let text = format!(
                    "title: {}\nmessage: {}\ntimes: {}",
                    event.get_title(),
                    event.get_message(),
                    entry.count,
                );

                let mut params = HashMap::new();
                params.insert("text", text);

                let client = reqwest::blocking::Client::builder()
                    .timeout(Duration::from_secs(5))
                    .build()
                    .unwrap();

                let res = client
                    .post(&self.slack_incoming_webhook_endpoint)
                    .json(&params)
                    .send();

                match res {
                    Ok(_) => {
                        entry.notified_at = Utc::now();
                        ()
                    }
                    Err(x) => error!("Notification error: {:?}", x),
                }
            } else {
                trace!("Suppresssed notification of {:?}", key);
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

struct NotificationEntry {
    pub notified_at: DateTime<Utc>,
    pub count: u64,
}

impl NotificationEntry {
    pub fn new() -> NotificationEntry {
        NotificationEntry {
            notified_at: Utc::now(),
            count: 0,
        }
    }
}
