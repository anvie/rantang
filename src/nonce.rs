use std::time::SystemTime;

pub fn nonce() -> u64 {
    let now = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap();
    let timestamp = now.as_secs();
    timestamp / 30
}
