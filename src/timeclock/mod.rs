
extern crate chrono;
extern crate rustc_serialize;

pub mod daterecord;
pub mod direction;
pub mod error;
pub mod timeentry;
mod timeentrypair;

use chrono::*;
use csv;
use self::daterecord::DateRecord;
use self::direction::Direction;
use self::error::TimeClockError;
pub use self::timeentry::TimeEntry;
pub use self::timeentrypair::{TimeEntryPair, TimeEntryPairsIter,
                              timeentry_pairs};
use std::collections::BTreeMap;
use std::fs::File;
use std::io::SeekFrom;
use std::io::prelude::*;

pub fn read_timesheet<R: Read>(file: R)
                               -> Result<Vec<TimeEntry>, TimeClockError> {
    let mut rdr = csv::Reader::from_reader(file).has_headers(false);
    let mut in_v = rdr.decode().collect::<csv::Result<Vec<TimeEntry>>>()?;
    in_v.sort_by_key(|k| k.time);

    return Ok(in_v);
}

/// Reduce pairs of TimeEntrys into DateRecords
pub fn collect_date_records(records: Vec<TimeEntry>) -> Vec<DateRecord> {
    let mut date_duration_map = BTreeMap::new();

    for tep in timeentry_pairs(records.into_iter()) {
        let r = DateRecord::from(tep);

        if !date_duration_map.contains_key(&r.date()) {
            date_duration_map.insert(r.date(), r);
        } else {
            let mut rec = date_duration_map.remove(&r.date()).unwrap();
            rec.add_seconds(r.seconds());
            rec.append_memo(&r.memo());

            date_duration_map.insert(rec.date(), rec);
        }
    }

    let v: Vec<DateRecord> = date_duration_map.iter()
        .map(|(_, r)| r.clone())
        .collect();

    return v;
}


/// Marks the time.
pub fn mark_time(d: Direction,
                 time: DateTime<FixedOffset>,
                 memo: String,
                 mut file: File) {
    let record = TimeEntry {
        dir: d,
        time: time,
        memo: memo,
    };
    // seek in case we write without reading first
    let _ = file.seek(SeekFrom::End(0));
    let mut wtr = csv::Writer::from_writer(file);
    let _ = wtr.encode(record);
}

/// Get the current date and time as a DateTime<FixedOffset>
pub fn now() -> DateTime<FixedOffset> {
    let lt: DateTime<Local> = Local::now();
    DateTime::parse_from_rfc3339(lt.to_rfc3339().as_ref()).unwrap()
}


#[cfg(test)]
mod tests {
    use std::io::Cursor;
    use super::*;
    use timeclock::direction::Direction;

    #[test]
    fn read_timesheet_test() {
        // test for sorting and general function
        let s = "Out,2016-12-18T16:53:33-0600,\n\
                 In,2016-12-19T20:54:53-0600,\n\
                 In,2016-12-18T13:01:50-0600,\n\
                 Out,2016-12-19T20:54:57-0600,";
        let vs = s.as_bytes();
        let buff = Cursor::new(vs);
        let records = read_timesheet(buff).unwrap();
        for slc in records.chunks(2) {
            assert!(slc[0].dir == Direction::In);
            assert!(slc[1].dir == Direction::Out);
        }
        // test ordering
        assert!(records[0].time < records[1].time);
        assert!(records[1].time < records[2].time);
        assert!(records[2].time < records[3].time);
    }


    #[test]
    #[should_panic(expected = "called `Result::unwrap()` on an `Err` value")]
    fn read_timesheet_haggis_test() {
        // test for sorting and general function
        let s = "Fair fa' your honest, sonsie face,\n\
                 Great chieftain o' the pudding-race!\n\
                 Aboon them a' ye tak your place,\n\
                 Painch, tripe, or thairm:\n\
                 Weel are ye wordy o' a grace\n\
                 As lang's my arm.";
        let vs = s.as_bytes();
        let buff = Cursor::new(vs);
        let records = read_timesheet(buff).unwrap();
        assert!(records[0].dir == Direction::In);
    }


    #[test]
    fn pair_time_entries_test() {
        let s = "Out,2016-12-18T13:01:50-0600,\n\
                 In,2016-12-19T20:54:57-0600,\n\
                 Out,2016-12-20T20:50:57-0600,\n\
                 Out,2016-12-20T20:50:59-0600,\n\
                 In,2016-12-20T21:04:57-0600,";
        let vs = s.as_bytes();
        let buff = Cursor::new(vs);
        let raw_records = read_timesheet(buff).unwrap();
        let records = pair_time_entries(raw_records);

        println!("\nlen={}", records.len());
        for record in records.clone() {
            println!("{}", record);
        }

        for slc in records.chunks(2) {
            assert!(slc[0].dir == Direction::In);
            assert!(slc[1].dir == Direction::Out);
        }
        assert!(records.len() == 8);
    }


    #[test]
    fn pair_time_entries_double_in_test() {
        let s = "In,2016-12-20T21:01:57-0600,\n\
                 In,2016-12-20T21:04:57-0600,";
        let vs = s.as_bytes();
        let buff = Cursor::new(vs);
        let raw_records = read_timesheet(buff).unwrap();
        let records = pair_time_entries(raw_records);

        println!("\nlen={}", records.len());
        for record in records.clone() {
            println!("{}", record);
        }

        for slc in records.chunks(2) {
            assert!(slc[0].dir == Direction::In);
            assert!(slc[1].dir == Direction::Out);
        }
        assert!(records.len() == 4);
    }


    #[test]
    fn collect_date_records_test() {
        let time = now();
        let records = vec![TimeEntry::new(Direction::In, time, "In"),
                           TimeEntry::new(Direction::Out, time, "Out")];

        let res = collect_date_records(records);
        assert_eq!(res.len(), 1);
        let dr = res.first().unwrap();
        assert_eq!(dr.date(), time.date());
        assert_eq!(dr.hours(), 0.0);
    }

    #[test]
    fn collect_date_records_tz_test() {
        // test mismatched timezones
        let s = "In,2016-12-18T13:01:50-0500,\n\
                 Out,2016-12-18T13:01:50-0600,";
        let vs = s.as_bytes();
        let buff = Cursor::new(vs);
        let entries = read_timesheet(buff).unwrap();
        let paired_records = pair_time_entries(entries);
        let records = collect_date_records(paired_records);

        assert!(records.len() == 1);
        println!("\n{}", records[0].seconds());
        assert!(records[0].seconds() == 3600.0);
    }
}
