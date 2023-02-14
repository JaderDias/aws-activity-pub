use std::convert::TryInto;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::SystemTime;
use std::time::UNIX_EPOCH;

static SEQUENCE_NUMBER: AtomicUsize = AtomicUsize::new(0);

pub fn get_id(node_id: u64) -> u64 {
    let snowflake_id_encoder = flaken::Flaken::default();
    let since_unix = SystemTime::now().duration_since(UNIX_EPOCH).unwrap();
    let ts: u64 = since_unix.as_millis().try_into().unwrap();
    let sequence_number: u64 = SEQUENCE_NUMBER
        .fetch_add(1, Ordering::SeqCst)
        .try_into()
        .unwrap();
    snowflake_id_encoder.encode(ts, node_id, sequence_number)
}
