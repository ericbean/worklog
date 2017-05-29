

use chrono::prelude::*;
use std::fmt;
use timeentry::TimeEntry;
use timeentry::TimeEntryPair;
use traits::ClockEntry;
use traits::Combine;
use traits::TimeRecord;

#[derive(Clone,Debug)]
pub struct DateRecord {
    date: Date<FixedOffset>,
    duration: f64,
    memo: String,
    complete: bool,
}


impl DateRecord {
    #[allow(dead_code)]
    /// Construct a DateRecord from it's constituent parts
    pub fn from_parts(date: Date<FixedOffset>, dur: f64, memo: &str) -> Self {
        DateRecord {
            date: date,
            duration: dur,
            memo: memo.to_owned(),
            complete: false,
        }
    }

    #[allow(dead_code)]
    // add seconds to the duration
    pub fn add_seconds(&mut self, secs: f64) {
        self.duration += secs;
    }

    /// Append &str to the memo
    pub fn append_memo(&mut self, memo: &str) {
        if !self.memo.is_empty() && !memo.is_empty() {
            self.memo.push_str(", ");
            self.memo.push_str(memo);
        } else if self.memo.is_empty() {
            self.memo.push_str(memo);
        }
    }
}


impl Combine for DateRecord {
    fn combine(&mut self, other: &Self) -> bool {
        if self.date == other.date {
            self.duration += other.duration;
            self.append_memo(other.memo());
            self.complete &= other.complete;
            true
        } else {
            false
        }
    }
}

impl TimeRecord<TimeEntry> for DateRecord {
    fn new(s: TimeEntry, e: TimeEntry, complete: bool) -> Self {
        let mut m = String::new();
        m.push_str(s.memo());
        if !s.memo().is_empty() && !e.memo().is_empty() {
            m.push_str(", ");
            m.push_str(e.memo());
        } else {
            m.push_str(e.memo());
        }
        DateRecord {
            date: s.time().date(),
            duration: e.time()
                .signed_duration_since(s.time())
                .num_seconds() as f64,
            memo: m,
            complete: complete,
        }
    }

    fn complete(&self) -> bool {
        self.complete
    }

    fn date(&self) -> Date<FixedOffset> {
        self.date
    }

    fn duration(&self) -> f64 {
        self.duration
    }

    fn memo(&self) -> &str {
        &self.memo
    }
}

impl fmt::Display for DateRecord {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let date = self.date.format("%F");
        write!(f, "{} {:.2} {}", date, self.duration / 3600.0, self.memo)
    }
}

impl From<TimeEntryPair> for DateRecord {
    fn from(tep: TimeEntryPair) -> Self {
        DateRecord {
            date: tep.date(),
            duration: tep.duration(),
            memo: tep.memo().to_owned(),
            complete: tep.complete(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use traits::Combine;

    const DURATION: f64 = 4321.098765;

    fn daterecord_helper(date: &str) -> DateRecord {
        let x = NaiveDate::parse_from_str(date, "%F").unwrap();
        let y = FixedOffset::west(6 * 3600).from_local_date(&x).unwrap();
        DateRecord::from_parts(y, DURATION, "Test")
    }

    #[test]
    fn duration_methods_test() {
        let mut dr = daterecord_helper("2017-01-07");
        // check math
        assert!(dr.duration() == DURATION);
        // check that addition works
        dr.add_seconds(1357.246);
        assert!(dr.duration() == DURATION + 1357.246);
    }

    #[test]
    fn memo_methods_test() {
        let mut dr = daterecord_helper("2017-01-07");
        assert!(dr.memo() == "Test");
        // append empty str
        dr.append_memo("");
        assert!(dr.memo() == "Test");
        // append actual str
        dr.append_memo("success!");
        assert!(dr.memo() == "Test, success!");
    }

    #[test]
    fn combine_test() {
        let mut a = daterecord_helper("2017-01-07");
        let b = daterecord_helper("2017-01-07");
        let success = a.combine(&b);
        assert!(success);
        assert_eq!(a.date(), a.date());
        assert_eq!(a.duration(), DURATION * 2.0);
        assert_eq!(a.memo(), "Test, Test");

        // a and c have different dates
        let c = daterecord_helper("2017-01-08");
        let success = a.combine(&c);
        assert!(!success);
        assert_eq!(a.date(), a.date());
        assert_eq!(a.duration(), DURATION * 2.0);
        assert_eq!(a.memo(), "Test, Test");
    }

    #[test]
    fn fmt_display_test() {
        let dr = daterecord_helper("2017-01-07");
        let s = format!("{}", dr);
        assert!(s == "2017-01-07 1.20 Test")
    }
}
