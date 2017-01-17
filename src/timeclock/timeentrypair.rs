use timeclock::TimeEntry;

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
