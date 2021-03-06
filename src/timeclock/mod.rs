
mod daterecord;
mod direction;
mod error;
mod timeentry;
mod traits;
mod iterators;

pub use self::daterecord::DateRecord;
pub use self::direction::Direction;
pub use self::error::TimeClockError;
pub use self::iterators::*;
pub use self::timeentry::{TimeEntry, TimeEntryPair};
pub use self::traits::*;
use chrono::*;
use csv;
use std::io::SeekFrom;
use std::io::prelude::*;

pub fn read_timesheet<R: Read>(file: R)
                               -> Result<Vec<TimeEntry>, TimeClockError> {
    let mut rdr = csv::ReaderBuilder::new()
        .has_headers(false)
        .from_reader(file);
    let mut in_v: Vec<TimeEntry> = try!(rdr.deserialize().collect());
    in_v.sort_by_key(|k| k.time);
    Ok(in_v)
}

/// Reduce pairs of `TimeEntrys` into `DateRecords`
pub fn collect_date_records(records: Vec<TimeEntry>) -> Vec<DateRecord> {
    let mut res: Vec<DateRecord> = Vec::new();
    for rec in timeentry_pairs(records.into_iter()).daterecords() {
        match res.pop() {
            Some(mut r) => {
                if r.combine(&rec) {
                    res.push(r);
                } else {
                    res.push(r);
                    res.push(rec);
                }
            }
            None => res.push(rec),
        }
    }
    res
}


/// Marks the time.
pub fn mark_time<W: Write + Seek>(dir: Direction,
                                  time: DateTime<FixedOffset>,
                                  memo: &str,
                                  file: &mut W) {
    let record = TimeEntry::new(dir, time, memo);
    // seek in case we write without reading first
    let _ = file.seek(SeekFrom::End(0));
    let mut wtr = csv::WriterBuilder::new()
        .has_headers(false)
        .from_writer(file);
    let _ = wtr.serialize(record);
}

/// Get the current date and time as a `DateTime`<`FixedOffset`>
pub fn now() -> DateTime<FixedOffset> {
    let lt: DateTime<Local> = Local::now();
    lt.with_timezone(lt.offset())
}


#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn read_timesheet_test() {
        // test for sorting and general function
        let s = "Out,2016-12-18T16:53:33-0600,\n\
                 In,2016-12-19T20:54:53-0600,\n\
                 In,2016-12-18T13:01:50-0600,\n\
                 Out,2016-12-19T20:54:57-0600,";
        let buff = Cursor::new(s.as_bytes());
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
    fn read_timesheet_haggis_test() {
        // test for sorting and general function
        let s = "Fair fa' your honest, sonsie face,\n\
                 Great chieftain o' the pudding-race!\n\
                 Aboon them a' ye tak your place,\n\
                 Painch, tripe, or thairm:\n\
                 Weel are ye wordy o' a grace\n\
                 As lang's my arm.";
        let buff = Cursor::new(s.as_bytes());
        let records = read_timesheet(buff);
        assert!(records.is_err());
    }


    #[test]
    fn pair_time_entries_test() {
        let s = "Out,2016-12-18T13:01:50-0600,\n\
                 In,2016-12-19T20:54:57-0600,\n\
                 Out,2016-12-20T20:50:57-0600,\n\
                 Out,2016-12-20T20:50:59-0600,\n\
                 In,2016-12-20T21:04:57-0600,";
        let buff = Cursor::new(s.as_bytes());
        let entries = read_timesheet(buff).unwrap();
        let records = timeentry_pairs(entries.into_iter())
            .collect::<Vec<TimeEntryPair>>();

        assert!(records.len() == 4);

        for tep in records {
            assert!(tep.start().dir == Direction::In);
            assert!(tep.end().dir == Direction::Out);
        }
    }


    #[test]
    fn pair_time_entries_double_in_test() {
        let s = "In,2016-12-20T21:01:57-0600,\n\
                 In,2016-12-20T21:04:57-0600,";
        let buff = Cursor::new(s.as_bytes());
        let entries = read_timesheet(buff).unwrap();
        let records = timeentry_pairs(entries.into_iter())
            .collect::<Vec<TimeEntryPair>>();

        assert!(records.len() == 2);

        for tep in records {
            assert!(tep.start().dir == Direction::In);
            assert!(tep.end().dir == Direction::Out);
        }
    }


    #[test]
    fn collect_date_records_test() {
        let time = now();
        let day = Duration::days(1);
        let records = vec![TimeEntry::new(Direction::In, time, "In"),
                           TimeEntry::new(Direction::Out, time, "Out"),
                           TimeEntry::new(Direction::In, time, "In"),
                           TimeEntry::new(Direction::Out, time, "Out"),
                           TimeEntry::new(Direction::In, time + day, "In"),
                           TimeEntry::new(Direction::Out, time + day, "Out")];

        let res = collect_date_records(records);
        assert_eq!(res.len(), 2);
        let dr = res.first().unwrap();
        assert_eq!(dr.date(), time.date());
        assert_eq!(dr.hours(), 0.0);
        let dr = res.last().unwrap();
        assert_eq!(dr.date(), time.date() + day);
        assert_eq!(dr.hours(), 0.0);
    }

    #[test]
    fn collect_date_records_tz_test() {
        // test mismatched timezones
        let s = "In,2016-12-18T13:01:50-0500,\n\
                 Out,2016-12-18T13:01:50-0600,";
        let buff = Cursor::new(s.as_bytes());
        let entries = read_timesheet(buff).unwrap();
        let records = collect_date_records(entries);

        assert!(records.len() == 1);
        println!("\n{}", records[0].seconds());
        assert!(records[0].seconds() == 3600.0);
    }

    #[test]
    fn mark_time_test() {
        let mut buff: Cursor<Vec<u8>> = Cursor::new(Vec::new());
        let time = DateTime::parse_from_rfc3339("2017-01-18T12:50:13-06:00")
            .unwrap();
        mark_time(Direction::In, time, "Test", &mut buff);
        let v = buff.into_inner();
        let s = String::from_utf8(v).unwrap();
        assert_eq!(s, "In,2017-01-18T12:50:13-06:00,Test\n");
    }
}
