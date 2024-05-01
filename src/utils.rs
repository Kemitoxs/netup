use std::fs::File;

use chrono::{TimeZone, Utc};
use csv::Writer;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::gui::TrackedMessage;

pub fn get_timestamp() -> u128 {
    use std::time::{SystemTime, UNIX_EPOCH};

    let start = SystemTime::now();
    let since_the_epoch = start
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards");
    since_the_epoch.as_millis()
}

pub fn format_timestamp_ms(epoch_ms: u128) -> String {
    // Convert the u128 milliseconds since the Unix epoch to a DateTime<Utc>
    let timestamp = Utc.timestamp_millis_opt(epoch_ms as i64).unwrap();

    // Format the DateTime object as "DD/MM/YY HH:MM:SS.mmm"
    timestamp.format("%d/%m/%y %H:%M:%S%.3f").to_string()
}

#[derive(Serialize, Deserialize, Debug, Hash)]
pub struct RawMessage {
    pub idx: u64,
    pub timestamp: u128,
    pub hash: u128,
}

impl RawMessage {
    pub fn build(idx: u64, timestamp: u128) -> Self {
        let mut msg = RawMessage {
            idx,
            timestamp,
            hash: 0,
        };
        msg.set_hash();
        msg
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        bincode::serialize(&self).expect("Failed to serialize message")
    }

    pub fn from_bytes(bytes: Vec<u8>) -> Option<Self> {
        bincode::deserialize(&bytes).ok()
    }

    fn calc_hash(&self) -> u128 {
        let mut hasher = Sha256::new();
        let data = format!("{}{}", self.idx, self.timestamp);
        let bytes = data.as_bytes();

        hasher.update(bytes);
        let result = hasher.finalize();
        let mut num_buf = [0 as u8; 16];
        num_buf.copy_from_slice(&result[..16]);
        let hash_num = u128::from_ne_bytes(num_buf);

        hash_num
    }

    pub fn check_hash(&self) -> bool {
        let hash_num = self.calc_hash();
        self.hash == hash_num
    }

    pub fn set_hash(&mut self) {
        self.hash = self.calc_hash();
    }
}

#[cfg(test)]
mod tests {
    use crate::gui::TrackedMessage;

    use super::SortedMessages;

    #[test]
    pub fn add_messages() {
        let mut messages = SortedMessages::default();
        let m1 = TrackedMessage::new(1, 1);
        messages.add(m1);
        let m2 = TrackedMessage::new(2, 3);
        messages.add(m2);
    }

    #[test]
    #[should_panic]
    pub fn add_invalid_messages() {
        let mut messages = SortedMessages::default();
        let m1 = TrackedMessage::new(1, 1);
        messages.add(m1);
        let m2 = TrackedMessage::new(2, 3);
        messages.add(m2);

        let m3 = TrackedMessage::new(3, 2);
        messages.add(m3);
    }

    #[test]
    pub fn find_upper_bound_message() {
        let mut messages = SortedMessages::default();
        let m1 = TrackedMessage::new(1, 1);
        messages.add(m1);
        let m2 = TrackedMessage::new(2, 3);
        messages.add(m2);
        let m3 = TrackedMessage::new(3, 5);
        messages.add(m3);

        let idx = messages.find(4, super::FindMode::UpperBound);
        assert_eq!(idx, Some(2));
    }

    #[test]
    pub fn find_lower_bound_message() {
        let mut messages = SortedMessages::default();
        let m1 = TrackedMessage::new(1, 1);
        messages.add(m1);
        let m2 = TrackedMessage::new(2, 3);
        messages.add(m2);
        let m3 = TrackedMessage::new(3, 5);
        messages.add(m3);

        let idx = messages.find(4, super::FindMode::LowerBound);
        assert_eq!(idx, Some(1));
    }

    #[test]
    pub fn find_exact_message() {
        let mut messages = SortedMessages::default();
        let m1 = TrackedMessage::new(1, 1);
        messages.add(m1);
        let m2 = TrackedMessage::new(2, 3);
        messages.add(m2);
        let m3 = TrackedMessage::new(3, 5);
        messages.add(m3);

        let idx = messages.find(3, super::FindMode::Exact);
        assert_eq!(idx, Some(1));
    }
}

pub struct SortedMessages {
    pub messages: Vec<TrackedMessage>,
    last_saved: u128,
}

impl Default for SortedMessages {
    fn default() -> Self {
        SortedMessages {
            messages: Vec::new(),
            last_saved: 0,
        }
    }
}

#[derive(PartialEq)]
pub enum FindMode {
    Exact,
    LowerBound,
    UpperBound,
}

impl SortedMessages {
    /// Adds a message to the list of messages, keeping the list sorted by snt_time
    pub fn add(&mut self, msg: TrackedMessage) {
        if self.messages.is_empty() {
            self.messages.push(msg);
            return;
        }

        let last = self.messages.last().unwrap();
        assert!(msg.snt_time > last.snt_time, "Messages are not sorted");

        self.messages.push(msg);
    }

    pub fn update(&mut self, rcv_time: u128) {
        todo!();
    }

    /// Uses binary search to find the message with the given snt_time
    pub fn find(&self, snt_time: u128, mode: FindMode) -> Option<usize> {
        let mut lower = 0;
        let mut upper = self.messages.len();

        loop {
            let range = upper - lower;

            if range <= 1 {
                return match mode {
                    FindMode::Exact => None,
                    FindMode::LowerBound => Some(lower),
                    FindMode::UpperBound => Some(upper),
                };
            }

            let idx;
            if range % 2 == 0 {
                idx = lower + range / 2;
            } else {
                idx = match mode {
                    FindMode::UpperBound => lower + range / 2 + 1,
                    _ => lower + range / 2,
                };
            }

            let item = &self.messages[idx];
            if item.snt_time == snt_time {
                return Some(idx);
            }

            if item.snt_time > snt_time {
                upper = idx;
            } else {
                lower = idx;
            }

            if lower == upper {
                return Some(lower);
            }
        }
    }

    pub fn get(&self, lower: u128, upper: u128) -> &[TrackedMessage] {
        let lower_idx = self.find(lower, FindMode::LowerBound).unwrap();
        let upper_idx = self.find(upper, FindMode::UpperBound).unwrap();

        &self.messages[lower_idx..upper_idx]
    }

    /// Writes the messages to a CSV file
    ///
    /// If `append` is true, the messages will be appended to the file
    /// If `append` is false, the file will be overwritten, a header will be written and all messages will be written
    pub fn write_to_file(&mut self, file: &mut File, append: bool) {
        let mut writer = Writer::from_writer(file);

        if !append {
            writer
                .write_record(&["idx", "snt_time", "rcv_time"])
                .expect("Failed to write header");
        }

        let messages_to_write = {
            let idx = self.find(self.last_saved, FindMode::UpperBound).unwrap();
            &self.messages[idx..]
        };

        if messages_to_write.is_empty() {
            return;
        }

        for msg in messages_to_write {
            writer.serialize(msg).expect("Failed to write message");
        }

        writer.flush().expect("Failed to flush writer");
        self.last_saved = self.messages.last().unwrap().snt_time;
    }

    pub fn load_from_file(&mut self, file: &mut File) {
        self.messages.clear();
    }
}
