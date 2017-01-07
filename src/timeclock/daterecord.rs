

use chrono::*;
use std::fmt;
use timeclock::timeentry::TimeEntry;


#[derive(Clone,Debug)]
pub struct DateRecord {
    date_: Date<FixedOffset>,
    duration: f64,
    memo_: String,
}


impl DateRecord {
    pub fn from_time_entries(start: &TimeEntry, end: &TimeEntry) -> DateRecord {
        let mut dr = DateRecord {
            date_: start.time.date(),
            duration: (end.time - start.time).num_seconds() as f64,
            memo_: String::new(),
        };
        dr.append_memo(&start.memo);
        dr.append_memo(&end.memo);
        return dr;
    }

    /// Get the duration, expressed in seconds
    pub fn seconds(&self) -> f64 {
        self.duration
    }


    #[allow(dead_code)]
    /// Get the duration, expressed in minutes
    pub fn minutes(&self) -> f64 {
        self.duration / 60.0
    }


    /// Get the duration, expressed in hours
    pub fn hours(&self) -> f64 {
        self.duration / 3600.0
    }


    pub fn date(&self) -> Date<FixedOffset> {
        self.date_
    }


    pub fn add_seconds(&mut self, secs: f64) {
        self.duration += secs;
    }


    /// Returns the memo
    pub fn memo(&self) -> &str {
        &self.memo_
    }


    /// Append &str to the memo
    pub fn append_memo(&mut self, memo: &str) {
        if !self.memo_.is_empty() && !memo.is_empty() {
            self.memo_.push_str(", ");
            self.memo_.push_str(memo);
        } else if self.memo_.is_empty() {
            self.memo_.push_str(memo);
        }
    }
}


impl fmt::Display for DateRecord {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{} {:.2} {}", self.date(), self.hours(), self.memo())
    }
}
