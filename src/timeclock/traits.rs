use chrono::prelude::*;
use std::iter::Iterator;
use timeclock::DateRecord;
use timeclock::DateRecordIter;

pub trait Combine<T = Self> {
    fn combine(&mut self, other: &T) -> bool;
}

pub trait TimeRecord {
    fn complete(&self) -> bool;
    fn date(&self) -> Date<FixedOffset>;
    fn duration(&self) -> f64;
    fn memo(&self) -> &str;
}

pub trait IntoDateRecords
    where Self: Sized,
          Self: Iterator,
          Self::Item: Into<DateRecord>
{
    fn daterecords(self) -> DateRecordIter<Self>;
}
