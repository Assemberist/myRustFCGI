use std::{sync::atomic::{AtomicU8, Ordering}, u8, usize};

// Task status
const TASK_STATUS_SIZE: usize = 2;

pub const STATUS_STOPPED: u8 = 0;
pub const STATUS_STARTED: u8 = 1;
pub const STATUS_SUCCEED: u8 = 2;
pub const STATUS_FAILED:  u8 = 3;

// Task types
const AMOUNT_OF_TASKS: usize = 7;

pub struct PlannerState {
    // there is 7 task types. each using 2 bit for status packed into u8.
    flags: [AtomicU8; AMOUNT_OF_TASKS / (8 / TASK_STATUS_SIZE) + 1],
    last_executed_task: usize,
}

impl PlannerState {
    fn get_task_status(&self, task: u8) -> (usize, u8, usize) {
        let byte = task as usize / 4;
        let data = self.flags[byte].load(Ordering::Acquire);
        let shift = (task as usize % 4) * 2;
        let value = (data >> shift) & 3;

        (byte, value, shift)
    }

    pub fn start_task(&self, task: u8) {
        let (byte, value, shift) = self.get_task_status(task);

        match value {
            // 00 -> 01
            STATUS_STOPPED => {
                let mask = 1 << shift;
                self.flags[byte].fetch_or(mask, Ordering::Acquire);
            },
            // 10 -> 01
            STATUS_SUCCEED => {
                let mask = 3 << shift;
                self.flags[byte].fetch_xor(mask, Ordering::Acquire);
            },
            // 11 -> 01
            STATUS_FAILED => {
                let mask = 2 << shift;
                self.flags[byte].fetch_xor(mask, Ordering::Acquire);
            },
            // 01 skipped
            _ => ()
        }
    }

    pub fn stop_task(&self, task: u8) -> bool {
        let (byte, value, shift) = self.get_task_status(task);

        match value {
            // 00 already stopped
            STATUS_STOPPED => true,
            // 10 -> 00
            STATUS_SUCCEED => {
                let mask = 2 << shift;
                self.flags[byte].fetch_xor(mask, Ordering::Acquire);
                true
            },
            // 11 -> 00
            STATUS_FAILED => {
                let mask = 3 << shift;
                self.flags[byte].fetch_xor(mask, Ordering::Acquire);
                true
            },
            // 01 skipped (task should be done first)
            _ => false,
        }
    }

    pub fn set_task_result(&self, task: u8, result: u8) {
        assert!(result == STATUS_SUCCEED || result == STATUS_FAILED);

        let (byte, value, shift) = self.get_task_status(task);

        match value {
            STATUS_STARTED => {
                // 01 -> 10 == 01 ^ 11 |  => 01 ^ (res ^ 01)
                // 01 -> 11 == 01 ^ 10 |
                let mask = (result ^ 1) << shift;
                self.flags[byte].fetch_xor(mask, Ordering::Acquire);
            },
            //already done or stopped
            _ => ()
        }
    }

    pub fn get_next_task(&mut self) -> Option<usize> {
        let start_pos = (self.last_executed_task + 1) % AMOUNT_OF_TASKS;
        
        let search_range = (start_pos..AMOUNT_OF_TASKS).chain(0..start_pos);

        let mut current_byte = 0;
        let mut last_loaded_byte_idx = usize::MAX;

        for a in search_range {
            let byte_idx = a / 4;
            
            if byte_idx != last_loaded_byte_idx {
                current_byte = self.flags[byte_idx].load(Ordering::Acquire);
                last_loaded_byte_idx = byte_idx;
            }

            let shift = (a % 4) * 2;
            let value = (current_byte >> shift) & 0b11; // 0b11 - это 3
            
            if value == STATUS_STARTED {
                self.last_executed_task = a;
                return Some(a);
            }
        }

        None
    }

    pub fn new() -> Self {
        PlannerState {
            flags: core::array::from_fn(|_| AtomicU8::new(0)),
            last_executed_task: 0,
        }
    }
}