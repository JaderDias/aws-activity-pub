use std::time::{SystemTime, UNIX_EPOCH};

use rand::{thread_rng, Rng};
use std::convert::TryInto;
use std::sync::Mutex;

struct Sequence {
    sequence_number: u64,
    last_timestamp: u64,
}

static MUTEX: Mutex<Sequence> = Mutex::new(Sequence {
    sequence_number: 0,
    last_timestamp: 0,
});

const SEQUENCE_BITS: u64 = 12;
const SEQUENCE_MASK: u64 = (1 << SEQUENCE_BITS) - 1;

const NODE_BITS: u64 = 10;
const NODE_RANGE: u64 = 1 << NODE_BITS;
const NODE_MASK: u64 = NODE_RANGE - 1;
const NODE_SHIFT: u64 = SEQUENCE_BITS; // 12 bits

const EPOCH_MILLISECONDS_SHIFT: u64 = SEQUENCE_BITS + NODE_BITS; // 22 bits, the remaining 42 bits are reserved for the timestamp, lasting until year 2100

pub fn get_node_id() -> u64 {
    let mut rng = thread_rng();
    rng.gen_range(0..NODE_RANGE)
}

pub fn get_id(node_id: u64) -> u64 {
    get_id_with_timestamp(node_id, SystemTime::now())
}

fn get_id_with_timestamp(node_id: u64, time: SystemTime) -> u64 {
    let since_unix = time.duration_since(UNIX_EPOCH).unwrap();
    let timestamp: u64 = since_unix.as_millis().try_into().unwrap();
    let sequence_number = fetch_add_sequence(timestamp);
    (timestamp << EPOCH_MILLISECONDS_SHIFT)
        | ((node_id & NODE_MASK) << NODE_SHIFT)
        | (sequence_number & SEQUENCE_MASK)
}

fn fetch_add_sequence(timestamp: u64) -> u64 {
    let mut data = MUTEX.lock().unwrap();
    let sequence_number = if timestamp > data.last_timestamp {
        data.last_timestamp = timestamp;
        0
    } else {
        data.sequence_number
    };

    data.sequence_number = sequence_number + 1;
    sequence_number
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[test]
    fn test_get_id_with_timestamp() {
        // Arrange
        const INITIAL_TIME_OFFSET_MILLISECONDS: u64 = 0b10101010101010101010101010101010101;
        const INITIAL_TIME_OFFSET_SECONDS: u64 = INITIAL_TIME_OFFSET_MILLISECONDS / 1000;
        let initial_time_offset_nanos: u32 =
            ((INITIAL_TIME_OFFSET_MILLISECONDS - (INITIAL_TIME_OFFSET_SECONDS * 1000)) * 1_000_000)
                .try_into()
                .unwrap();
        const NODE_ID: u64 = 0b1100110000;
        let initial_time_offset =
            Duration::new(INITIAL_TIME_OFFSET_SECONDS, initial_time_offset_nanos);
        let initial_system_time = SystemTime::UNIX_EPOCH + initial_time_offset;

        // Act
        let first = get_id_with_timestamp(NODE_ID, initial_system_time);

        // Assert
        assert_eq!(
            first,
            // 42 bits of timestamp
            //                                          10 bits of node_id
            //                                                    12 bits of sequence number
            //TTTTTTTTTTTTTTTTTTTTTTTTTTTTTTTTTTTTTTTTTTNNNNNNNNNNSSSSSSSSSSSS
            0b0000000101010101010101010101010101010101011100110000000000000000,
        );

        // Act
        let second = get_id_with_timestamp(NODE_ID, initial_system_time);

        // Assert
        assert_eq!(
            second,
            // 42 bits of timestamp
            //                                          10 bits of node_id
            //                                                    12 bits of sequence number
            //TTTTTTTTTTTTTTTTTTTTTTTTTTTTTTTTTTTTTTTTTTNNNNNNNNNNSSSSSSSSSSSS
            0b0000000101010101010101010101010101010101011100110000000000000001,
        );

        // Arrange
        let new_time_offset =
            Duration::new(INITIAL_TIME_OFFSET_SECONDS + 1, initial_time_offset_nanos);
        let new_system_time = SystemTime::UNIX_EPOCH + new_time_offset;

        // Act
        let second = get_id_with_timestamp(NODE_ID, new_system_time);

        // Assert
        assert_eq!(
            second,
            // 42 bits of timestamp
            //                                          10 bits of node_id
            //                                                    12 bits of sequence number
            //TTTTTTTTTTTTTTTTTTTTTTTTTTTTTTTTTTTTTTTTTTNNNNNNNNNNSSSSSSSSSSSS
            0b0000000101010101010101010101011001001111011100110000000000000000,
        );
    }
}
