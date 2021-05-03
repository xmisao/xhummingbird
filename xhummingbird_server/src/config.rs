use std::env;

pub fn slack_incoming_webhook_endpoint() -> String {
    env::var("XH_SLACK_INCOMING_WEBHOOK_ENDPOINT")
        .unwrap()
        .to_string()
}

pub fn notification_threshold() -> u32 {
    let default_notification_threshold = "0".to_string();
    env::var("XH_NOTIFICATION_THRESHOLD")
        .unwrap_or(default_notification_threshold)
        .parse::<u32>()
        .unwrap()
}

pub fn no_control() -> bool {
    let default_no_control = "0".to_string();
    let no_control = env::var("XH_NO_CONTROL")
        .unwrap_or(default_no_control)
        .parse::<u32>()
        .unwrap();
    no_control != 0
}

pub fn snapshot() -> String {
    env::var("XH_SNAPSHOT").unwrap()
}
