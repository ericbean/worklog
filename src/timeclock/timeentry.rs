use chrono::*;
use std::fmt;
use timeclock::direction::Direction;

#[derive(Clone,Debug,PartialEq,Serialize,Deserialize)]
pub struct TimeEntry {
    pub dir: Direction,
    pub time: DateTime<FixedOffset>,
    pub memo: String,
}

impl TimeEntry {
    pub fn new(dir: Direction,
               time: DateTime<FixedOffset>,
               memo: &str)
               -> Self {
        TimeEntry {
            dir: dir,
            time: time,
            memo: memo.to_owned(),
        }
    }
}

impl fmt::Display for TimeEntry {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let time = self.time.format("%F %I:%M %P");
        f.write_fmt(format_args!("{:3} {} {}", self.dir, time, self.memo))
    }
}

#[derive(Debug,PartialEq)]
pub struct TimeEntryPair {
    start: TimeEntry,
    end: TimeEntry,
}

impl TimeEntryPair {
    pub fn new(s: TimeEntry, e: TimeEntry) -> Self {
        TimeEntryPair { start: s, end: e }
    }

    pub fn start(&self) -> &TimeEntry {
        &self.start
    }

    pub fn end(&self) -> &TimeEntry {
        &self.end
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use timeclock::Direction;

    fn time_helper() -> DateTime<FixedOffset> {
        DateTime::parse_from_rfc3339("2017-01-05T14:04:16-06:00").unwrap()
    }

    #[test]
    fn timeentry_print_test() {
        let time = DateTime::parse_from_rfc3339("2017-01-05T14:04:16-06:00")
            .unwrap();
        let te = TimeEntry::new(Direction::In, time, "Test");
        let display = format!("{}", te);
        assert_eq!(display, "In  2017-01-05 02:04 pm Test");
        let debug = format!("{:?}", te);
        assert_eq!(debug,
                   "TimeEntry { dir: In, time: 2017-01-05T14:04:16-06:00, memo: \"Test\" }");
    }

    #[test]
    fn timeentrypair_test() {
        let time = time_helper();
        let te_a = TimeEntry::new(Direction::In, time, "Test");
        let te_b = TimeEntry::new(Direction::Out, time, "Test");
        let tep = TimeEntryPair::new(te_a.clone(), te_b.clone());
        assert_eq!(tep.start(), &te_a);
        assert_eq!(tep.end(), &te_b);
    }
}
