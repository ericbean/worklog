

use chrono::*;
use std::fmt;
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
        return dr;
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

    fn daterecord_helper() -> DateRecord {
        let x = NaiveDate::parse_from_str("2017-01-07", "%F").unwrap();
        let y = FixedOffset::west(6 * 3600).from_local_date(&x).unwrap();
        DateRecord::from_parts(y, 4321.098765, "Test")
    }

    #[test]
    fn duration_methods_test() {
        let mut dr = daterecord_helper();
        // check math
        assert!(dr.seconds() == 4321.098765);
        assert!(dr.minutes() == 72.01831274999999);
        assert!(dr.hours() == 1.2003052125);
        // check that addition works
        dr.add_seconds(1357.246);
        assert!(dr.seconds() == 5678.344765);
    }

    #[test]
    fn memo_methods_test() {
        let mut dr = daterecord_helper();
        assert!(dr.memo() == "Test");
        // append empty str
        dr.append_memo("");
        assert!(dr.memo() == "Test");
        // append actual str
        dr.append_memo("success!");
        assert!(dr.memo() == "Test, success!");
    }

    #[test]
    fn fmt_debug_test() {
        let dr = daterecord_helper();
        let s = format!("{:?}", dr);
        println!("{:?}", dr);
        assert!(s ==
                "DateRecord { date: 2017-01-07-06:00, \
                duration: 4321.098765, memo: \"Test\" }")
    }

    #[test]
    fn fmt_display_test() {
        let dr = daterecord_helper();
        let s = format!("{}", dr);
        assert!(s == "2017-01-07 1.20 Test")
    }
}