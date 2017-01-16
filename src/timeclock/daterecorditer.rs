use std::iter::Iterator;
use timeclock::DateRecord;

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

#[cfg(test)]
mod tests {
    use super::*;
    use timeclock::Direction;
    use timeclock::TimeEntry;
    use timeclock::TimeEntryPair;
    use timeclock::now;

    #[test]
    fn date_record_iter_test() {
        let te_a = TimeEntry::new(Direction::In, now(), "Entry A");
        let te_b = TimeEntry::new(Direction::Out, now(), "Entry B");
        let tep = TimeEntryPair::new(te_a, te_b);
        let v = vec![tep];
        let driter = DateRecordIter { v: v.into_iter() };

        for dr in driter {
            assert!(dr.seconds() < 1000.0);
        }
    }
}