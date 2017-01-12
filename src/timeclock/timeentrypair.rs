use std::convert::From;
use std::iter::Iterator;
use std::mem;
use timeclock::direction::Direction;
use timeclock::timeentry::TimeEntry;

/// Option-like enclosure for TimeEntrys
#[derive(Debug,PartialEq)]
enum TimeEntryOpt {
    In(TimeEntry),
    Out(TimeEntry),
    Invalid,
}

impl TimeEntryOpt {
    /// Take the TimeEntry variant and leave Invalid in it's place
    pub fn take(&mut self) -> TimeEntryOpt {
        mem::replace(self, TimeEntryOpt::Invalid)
    }
}

impl From<Option<TimeEntry>> for TimeEntryOpt {
    fn from(opt: Option<TimeEntry>) -> Self {
        let te = match opt {
            Some(te) => te,
            None => {
                return TimeEntryOpt::Invalid;
            }
        };
        match te.dir {
            Direction::In => TimeEntryOpt::In(te),
            Direction::Out => TimeEntryOpt::Out(te),
        }
    }
}

impl From<TimeEntry> for TimeEntryOpt {
    fn from(te: TimeEntry) -> Self {
        match te.dir {
            Direction::In => TimeEntryOpt::In(te),
            Direction::Out => TimeEntryOpt::Out(te),
        }
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


/// Iterator to create TimeEntryPairs from TimeEntrys
pub struct TimeEntryPairsIter<I> {
    buf: TimeEntryOpt,
    v: I,
}

/// Constructor for TimeEntryPairsIter
pub fn timeentry_pairs<I>(entries: I) -> TimeEntryPairsIter<I>
    where I: Iterator<Item = TimeEntry>
{
    TimeEntryPairsIter {
        buf: TimeEntryOpt::Invalid,
        v: entries.into_iter(),
    }
}

impl<I> Iterator for TimeEntryPairsIter<I>
    where I: Iterator<Item = TimeEntry>
{
    type Item = TimeEntryPair;
    fn next(&mut self) -> Option<TimeEntryPair> {
        if self.buf == TimeEntryOpt::Invalid {
            self.buf = TimeEntryOpt::from(self.v.next());
        }

        let nxt = TimeEntryOpt::from(self.v.next());
        match (self.buf.take(), nxt) {
            (TimeEntryOpt::In(start), TimeEntryOpt::In(saved)) => {
                let mut end = start.clone();
                end.dir = Direction::Out;
                end.memo = String::from("Missing clock out.");
                self.buf = TimeEntryOpt::from(saved);
                Some(TimeEntryPair::new(start, end))
            }
            (TimeEntryOpt::In(start), TimeEntryOpt::Invalid) => {
                let mut end = start.clone();
                end.dir = Direction::Out;
                end.memo = String::from("Missing clock out.");
                Some(TimeEntryPair::new(start, end))
            }
            (TimeEntryOpt::In(start), TimeEntryOpt::Out(end)) => {
                Some(TimeEntryPair::new(start, end))
            }
            (TimeEntryOpt::Out(end), TimeEntryOpt::Out(saved)) => {
                let mut start = end.clone();
                start.dir = Direction::In;
                start.memo = String::from("Missing clock in.");
                self.buf = TimeEntryOpt::from(saved);
                Some(TimeEntryPair::new(start, end))
            }
            (TimeEntryOpt::Out(end), TimeEntryOpt::In(saved)) => {
                let mut start = end.clone();
                start.dir = Direction::In;
                start.memo = String::from("Missing clock in.");
                self.buf = TimeEntryOpt::from(saved);
                Some(TimeEntryPair::new(start, end))
            }
            (TimeEntryOpt::Out(end), TimeEntryOpt::Invalid) => {
                let mut start = end.clone();
                start.dir = Direction::In;
                start.memo = String::from("Missing clock in.");
                Some(TimeEntryPair::new(start, end))
            }
            (TimeEntryOpt::Invalid, TimeEntryOpt::Out(_)) => unreachable!(), 
            (TimeEntryOpt::Invalid, TimeEntryOpt::In(_)) => unreachable!(), 
            (TimeEntryOpt::Invalid, TimeEntryOpt::Invalid) => None,
        }
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use timeclock::direction::Direction;
    use timeclock::now;
    use timeclock::timeentry::TimeEntry;

    #[test]
    fn iter_in_in_test() {
        let a = TimeEntry::new(Direction::In, now(), "In");
        let b = TimeEntry::new(Direction::In, now(), "In");
        let v = vec![a, b];
        let y = timeentry_pairs(v.into_iter()).collect::<Vec<TimeEntryPair>>();

        assert_eq!(y.len(), 2);

        for pair in y {
            let (s, e) = (pair.start(), pair.end());
            assert_eq!(s.dir, Direction::In);
            assert_eq!(e.dir, Direction::Out);
        }
    }

    #[test]
    fn iter_in_invalid_test() {
        let a = TimeEntry::new(Direction::In, now(), "In");
        let v = vec![a];
        let y = timeentry_pairs(v.into_iter()).collect::<Vec<TimeEntryPair>>();

        assert_eq!(y.len(), 1);

        for pair in y {
            let (s, e) = (pair.start(), pair.end());
            assert_eq!(s.dir, Direction::In);
            assert_eq!(e.dir, Direction::Out);
        }
    }

    #[test]
    fn iter_in_out_test() {
        let a = TimeEntry::new(Direction::In, now(), "In");
        let b = TimeEntry::new(Direction::Out, now(), "Out");
        let v = vec![a, b];
        let y = timeentry_pairs(v.into_iter()).collect::<Vec<TimeEntryPair>>();

        assert_eq!(y.len(), 1);

        for pair in y {
            let (s, e) = (pair.start(), pair.end());
            assert_eq!(s.dir, Direction::In);
            assert_eq!(e.dir, Direction::Out);
        }
    }

    #[test]
    fn iter_out_out_test() {
        let a = TimeEntry::new(Direction::Out, now(), "Out");
        let b = TimeEntry::new(Direction::Out, now(), "Out");
        let v = vec![a, b];
        let y = timeentry_pairs(v.into_iter()).collect::<Vec<TimeEntryPair>>();

        assert_eq!(y.len(), 2);

        for pair in y {
            let (s, e) = (pair.start(), pair.end());
            assert_eq!(s.dir, Direction::In);
            assert_eq!(e.dir, Direction::Out);
        }
    }

    #[test]
    fn iter_out_in_test() {
        let a = TimeEntry::new(Direction::Out, now(), "Out");
        let b = TimeEntry::new(Direction::In, now(), "In");
        let v = vec![a, b];
        let y = timeentry_pairs(v.into_iter()).collect::<Vec<TimeEntryPair>>();

        assert_eq!(y.len(), 2);

        for pair in y {
            let (s, e) = (pair.start(), pair.end());
            assert_eq!(s.dir, Direction::In);
            assert_eq!(e.dir, Direction::Out);
        }
    }

    #[test]
    fn iter_out_invalid_test() {
        let a = TimeEntry::new(Direction::Out, now(), "Out");
        let v = vec![a];
        let y = timeentry_pairs(v.into_iter()).collect::<Vec<TimeEntryPair>>();

        assert_eq!(y.len(), 1);

        for pair in y {
            let (s, e) = (pair.start(), pair.end());
            assert_eq!(s.dir, Direction::In);
            assert_eq!(e.dir, Direction::Out);
        }
    }

    #[test]
    fn iter_invalid_invalid_test() {
        let v: Vec<TimeEntry> = Vec::with_capacity(0);
        let y = timeentry_pairs(v.into_iter()).collect::<Vec<TimeEntryPair>>();

        assert_eq!(y.len(), 0);
    }

}