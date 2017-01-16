

use chrono::*;
use std::fmt;
use timeclock::Combine;
use timeclock::TimeEntry;
use timeclock::TimeEntryPair;


#[derive(Clone,Debug)]
pub struct DateRecord {
    date: Date<FixedOffset>,
    duration: f64,
    memo: String,
}


impl DateRecord {
    pub fn from_time_entries(start: &TimeEntry, end: &TimeEntry) -> DateRecord {
        let mut dr = DateRecord {
            date: start.time.date(),
            duration: (end.time - start.time).num_seconds() as f64,
            memo: String::new(),
        };
        dr.append_memo(&start.memo);
        dr.append_memo(&end.memo);
        dr
    }


    #[allow(dead_code)]
    /// Construct a DateRecord from it's constituent parts
    pub fn from_parts(date: Date<FixedOffset>, dur: f64, memo: &str) -> Self {
        DateRecord {
            date: date,
            duration: dur,
            memo: memo.to_owned(),
        }
    }

    #[allow(dead_code)]
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
        self.date
    }

    #[allow(dead_code)]
    // add seconds to the duration
    pub fn add_seconds(&mut self, secs: f64) {
        self.duration += secs;
    }


    /// Returns the memo
    pub fn memo(&self) -> &str {
        &self.memo
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
            true
        } else {
            false
        }
    }
}


impl fmt::Display for DateRecord {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let date = self.date.format("%F");
        write!(f, "{} {:.2} {}", date, self.hours(), self.memo)
    }
}


impl From<TimeEntryPair> for DateRecord {
    fn from(tep: TimeEntryPair) -> Self {
        DateRecord::from_time_entries(tep.start(), tep.end())
    }
}

#[cfg(test)]
mod tests {
    use chrono::*;
    use super::*;
    use timeclock::Combine;

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
        assert!(dr.seconds() == DURATION);
        assert!(dr.minutes() == DURATION / 60.0);
        assert!(dr.hours() == DURATION / 3600.0);
        // check that addition works
        dr.add_seconds(1357.246);
        assert!(dr.seconds() == DURATION + 1357.246);
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
        assert_eq!(a.seconds(), DURATION * 2.0);
        assert_eq!(a.memo(), "Test, Test");

        // a and c have different dates
        let c = daterecord_helper("2017-01-08");
        let success = a.combine(&c);
        assert!(!success);
        assert_eq!(a.date(), a.date());
        assert_eq!(a.seconds(), DURATION * 2.0);
        assert_eq!(a.memo(), "Test, Test");
    }

    #[test]
    fn fmt_debug_test() {
        let dr = daterecord_helper("2017-01-07");
        let s = format!("{:?}", dr);
        println!("{:?}", dr);
        assert!(s ==
                "DateRecord { date: 2017-01-07-06:00, \
                duration: 4321.098765, memo: \"Test\" }")
    }

    #[test]
    fn fmt_display_test() {
        let dr = daterecord_helper("2017-01-07");
        let s = format!("{}", dr);
        assert!(s == "2017-01-07 1.20 Test")
    }
}