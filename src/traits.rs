use chrono::prelude::*;
use iterators::DateRecordIter;
use records::{DateRecord, Direction};
use std::iter::Iterator;

pub trait Combine<T = Self> {
    fn combine(&mut self, other: &T) -> bool;
}

pub trait ClockEntry {
    fn direction(&self) -> Direction;
    fn time(&self) -> DateTime<FixedOffset>;
    fn memo(&self) -> &str;
}

pub trait TimeRecord<T>
    where T: ClockEntry
{
    fn new(T, T, bool) -> Self;
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
