use std::time::{SystemTime, UNIX_EPOCH};

pub fn now() -> u64 {
    let start = SystemTime::now();
    let since_the_epoch = start.duration_since(UNIX_EPOCH);
    since_the_epoch.unwrap().as_millis() as u64
}

pub fn since(old_timestamp: u64) -> u64 {
    let millis = now();
    if millis > old_timestamp {
        millis - old_timestamp
    } else {
        0
    }
}
