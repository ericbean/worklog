use super::now;
use records::{DateRecord, TimeEntry, TimeEntryPair};
use records::Direction;
use std::iter::Iterator;
use std::mem;
use traits::ClockEntry;
use traits::IntoDateRecords;
use traits::TimeRecord;

/// Iterator adapter to create `DateRecords`
pub struct DateRecordIter<I> {
    v: I,
}

impl<I> DateRecordIter<I> {
    pub fn new(v: I) -> Self {
        DateRecordIter { v: v }
    }
}

impl<I> Iterator for DateRecordIter<I>
    where I: Iterator,
          I::Item: Into<DateRecord>
{
    type Item = DateRecord;
    fn next(&mut self) -> Option<Self::Item> {
        match self.v.next() {
            // Fucking magic.
            Some(tep) => Some(tep.into()),
            None => None,
        }
    }
}

/// Option-like enclosure for `TimeEntrys`
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
        match opt {
            Some(te) => te.into(),
            None => TimeEntryOpt::Invalid,
        }
    }
}

impl From<TimeEntry> for TimeEntryOpt {
    fn from(te: TimeEntry) -> Self {
        match te.direction() {
            Direction::In => TimeEntryOpt::In(te),
            Direction::Out => TimeEntryOpt::Out(te),
        }
    }
}

pub struct TimeEntryPairsIter<I> {
    buf: TimeEntryOpt,
    v: I,
}

/// Constructor for `TimeEntryPairsIter`
pub fn timeentry_pairs<I>(entries: I) -> TimeEntryPairsIter<I>
    where I: Iterator<Item = TimeEntry>
{
    TimeEntryPairsIter {
        buf: TimeEntryOpt::Invalid,
        v: entries.into_iter(),
    }
}

/// Iterator to create `TimeEntryPairs` from `TimeEntrys`
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
                let end = TimeEntry::new(Direction::Out, start.time(), "");
                self.buf = TimeEntryOpt::from(saved);
                Some(TimeEntryPair::new(start, end, false))
            }
            (TimeEntryOpt::In(start), TimeEntryOpt::Invalid) => {
                let end = TimeEntry::new(Direction::Out, now(), "");
                Some(TimeEntryPair::new(start, end, false))
            }
            (TimeEntryOpt::In(start), TimeEntryOpt::Out(end)) => {
                Some(TimeEntryPair::new(start, end, true))
            }
            (TimeEntryOpt::Out(end), TimeEntryOpt::In(saved)) |
            (TimeEntryOpt::Out(end), TimeEntryOpt::Out(saved)) => {
                let start = TimeEntry::new(Direction::In, end.time(), "");
                self.buf = TimeEntryOpt::from(saved);
                Some(TimeEntryPair::new(start, end, false))
            }
            (TimeEntryOpt::Out(end), TimeEntryOpt::Invalid) => {
                let start = TimeEntry::new(Direction::In, end.time(), "");
                Some(TimeEntryPair::new(start, end, false))
            }
            (TimeEntryOpt::Invalid, TimeEntryOpt::Out(_)) => unreachable!(), 
            (TimeEntryOpt::Invalid, TimeEntryOpt::In(_)) => unreachable!(), 
            (TimeEntryOpt::Invalid, TimeEntryOpt::Invalid) => None,
        }
    }
}

impl<I> IntoDateRecords for TimeEntryPairsIter<I>
    where I: Iterator<Item = TimeEntry>
{
    fn daterecords(self) -> DateRecordIter<Self> {
        DateRecordIter::new(self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use records::{TimeEntry, TimeEntryPair};
    use traits::IntoDateRecords;

    #[test]
    fn date_record_iter_test() {
        let te_a = TimeEntry::new(Direction::In, now(), "Entry A");
        let te_b = TimeEntry::new(Direction::Out, now(), "Entry B");
        let tep = TimeEntryPair::new(te_a, te_b, true);
        let v = vec![tep];
        let driter = DateRecordIter { v: v.into_iter() };

        for dr in driter {
            assert!(dr.duration() < 1000.0);
        }
    }

    #[test]
    fn iter_in_in_test() {
        let a = TimeEntry::new(Direction::In, now(), "In");
        let b = TimeEntry::new(Direction::In, now(), "In");
        let v = vec![a, b];
        let y = timeentry_pairs(v.into_iter()).collect::<Vec<TimeEntryPair>>();

        assert_eq!(y.len(), 2);

        for pair in y {
            let (s, e) = (pair.start(), pair.end());
            assert_eq!(s.direction(), Direction::In);
            assert_eq!(e.direction(), Direction::Out);
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
            assert_eq!(s.direction(), Direction::In);
            assert_eq!(e.direction(), Direction::Out);
            assert_eq!(e.memo(), "");
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
            assert_eq!(s.direction(), Direction::In);
            assert_eq!(e.direction(), Direction::Out);
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
            assert_eq!(s.direction(), Direction::In);
            assert_eq!(e.direction(), Direction::Out);
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
            assert_eq!(s.direction(), Direction::In);
            assert_eq!(e.direction(), Direction::Out);
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
            assert_eq!(s.direction(), Direction::In);
            assert_eq!(e.direction(), Direction::Out);
        }
    }

    #[test]
    fn iter_invalid_invalid_test() {
        let v: Vec<TimeEntry> = Vec::with_capacity(0);
        let y = timeentry_pairs(v.into_iter()).collect::<Vec<TimeEntryPair>>();

        assert_eq!(y.len(), 0);
    }

    #[test]
    fn to_daterecords() {
        let time = now();
        let v = vec![TimeEntry::new(Direction::In, time, "Entry A"),
                     TimeEntry::new(Direction::Out, time, "Entry B")];

        let tepiter = timeentry_pairs(v.into_iter());
        for dr in tepiter.daterecords() {
            assert_eq!(dr.duration(), 0.0);
        }
    }
}
