

use chrono::*;
use std::fmt;
use timeclock::timeentry::TimeEntry;


#[derive(Clone,Debug)]
pub struct DateRecord {
    pub date: Date<FixedOffset>,
    pub duration: Duration,
    pub memo: String,
}


impl DateRecord {
    pub fn from_time_entries(start: &TimeEntry, end: &TimeEntry) -> DateRecord {
        let mut temp_memo = String::new();
        if !start.memo.is_empty() {
            temp_memo.push_str(start.memo.trim());
        }
        if !end.memo.is_empty() {
            if !start.memo.is_empty() {
                temp_memo.push_str(", ");
            }
            temp_memo.push_str(end.memo.trim());
        }
        DateRecord {
            date: start.time.date(),
            duration: end.time - start.time,
            memo: temp_memo,
        }
    }

    /// Get the duration, expressed in seconds
    pub fn seconds(&self) -> f64 {
        self.duration.num_seconds() as f64
    }


    #[allow(dead_code)]
    /// Get the duration, expressed in minutes
    pub fn minutes(&self) -> f64 {
        self.seconds() / 60.0
    }


    /// Get the duration, expressed in hours
    pub fn hours(&self) -> f64 {
        self.seconds() / 3600.0
    }
}


impl fmt::Display for DateRecord {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{} {:.2} {}", self.date, self.hours(), self.memo)
    }
}
