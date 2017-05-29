// Worklog is for recording your hours.
//
extern crate chrono;
extern crate csv;
extern crate clap;
#[macro_use]
extern crate serde_derive;
extern crate serde;

mod error;
mod util;
mod parsers;
mod records;
mod traits;
mod iterators;

use chrono::*;
use clap::{Arg, ArgGroup, App};
use error::{TimeClockError, WorklogError};
use iterators::timeentry_pairs;
use records::{DateRecord, Direction, TimeEntry};
use std::env;
use std::fs::OpenOptions;
use std::io::SeekFrom;
use std::io::prelude::*;
use std::path::PathBuf;
use traits::*;

#[cfg(target_family = "unix")]
static CSV_FILE_NAME: &'static str = ".worklog.csv";
#[cfg(target_family = "windows")]
static CSV_FILE_NAME: &'static str = "worklog.csv";

// default rounding mode
static ROUNDING_DEFAULT: &'static str = "+15m";

// TODO This belongs in a config file somewhere
const WEEKSTART: i64 = Weekday::Sat as i64;


fn print_csv_entries<R: Read>(file: R) -> Result<(), WorklogError> {
    let csv_entries = try!(read_timesheet(file));
    for rec in csv_entries {
        println!("{}", rec);
    }
    Ok(())
}

fn print_summary<R: Read>(file: R,
                          start_date: Date<FixedOffset>,
                          end_date: Date<FixedOffset>,
                          rounding: util::Rounding)
                          -> Result<(), WorklogError> {
    let csv_entries = try!(read_timesheet(file));
    let records = collect_date_records(csv_entries);

    let mut total_hours: f64 = 0.0;
    for rec in records {
        if start_date <= rec.date() && rec.date() <= end_date {
            let hours = util::round(rec.duration(), rounding) / 3600.0;
            total_hours += hours;

            if rec.complete() {
                println!("{} {:.2} {}",
                         rec.date().format("%F"),
                         hours,
                         rec.memo());
            } else {
                println!("{} {:.2} {} Missing record(s)",
                         rec.date().format("%F"),
                         hours,
                         rec.memo());
            }
        }
    }

    println!("Total Hours: {:.2}", total_hours);
    Ok(())
}

/// Get the path for the csv data file
fn get_csv_path() -> Result<PathBuf, WorklogError> {
    let mut data_path = match env::home_dir() {
        Some(path) => path,
        None => {
            return Err(env::VarError::NotPresent).map_err(WorklogError::Env)
        }
    };

    data_path.push(CSV_FILE_NAME);
    Ok(data_path)
}

pub fn read_timesheet<R: Read>(file: R)
                               -> Result<Vec<TimeEntry>, TimeClockError> {
    let mut rdr = csv::ReaderBuilder::new()
        .has_headers(false)
        .from_reader(file);
    let mut in_v: Vec<TimeEntry> = try!(rdr.deserialize().collect());
    in_v.sort_by_key(|k| k.time());
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

fn main0() -> Result<(), WorklogError> {
    // Using std env macro rather than depending on clap's. No difference
    // as far as I can tell.
    // The current options aren't final and will definately change again soon
    let matches = App::new(env!("CARGO_PKG_NAME"))
        .version(env!("CARGO_PKG_VERSION"))
        .author(env!("CARGO_PKG_AUTHORS"))
        .about(env!("CARGO_PKG_DESCRIPTION"))
        .arg(Arg::from_usage("[in] -i, --in 'Record an In entry'"))
        .arg(Arg::from_usage("[out] -o, --out 'Record an Out entry'"))
        .arg(Arg::from_usage("[time] -t, --time <TIME> 'time'")
            .allow_hyphen_values(true)
            .requires("inout"))
        .arg(Arg::from_usage("[memo] -m, --memo <MEMO> 'Memo for the entry'")
            .requires("inout"))
        .arg(Arg::from_usage("[summary] -s, --summary 'Print a summary'")
            .conflicts_with("inout"))
        .arg(Arg::from_usage("[log] -l, --log 'Print the full log'")
            .conflicts_with("summary")
            .conflicts_with("inout"))
        .arg(Arg::from_usage("[round] -r, --round-up 'Round totals up to the next quarter hour'")
            .conflicts_with("log")
            .conflicts_with("inout"))
        .arg(Arg::from_usage("[round_ex] -R, --round <ROUNDING> 'Round totals up, down, half'")
            .allow_hyphen_values(true)
            .conflicts_with("log")
            .conflicts_with("inout"))
        .arg(Arg::from_usage("[range] --range <TIME> <TIME> 'range'")
            .conflicts_with("log")
            .conflicts_with("inout"))
        .group(ArgGroup::with_name("inout").args(&["in", "out"]))
        .get_matches();

    let csv_path = try!(get_csv_path());

    let mut csv_file = try!(OpenOptions::new()
                                .read(true)
                                .write(true)
                                .create(true)
                                .open(csv_path));

    let rounding = {
        if matches.occurrences_of("round") > 0 {
            try!(parsers::parse_rounding(ROUNDING_DEFAULT))
        } else if matches.occurrences_of("round_ex") > 0 {
            try!(parsers::parse_rounding(matches.value_of("round_ex").unwrap()))
        } else {
            util::Rounding::None
        }
    };

    let ctime = now();

    let (start_date, end_date): (Date<FixedOffset>, Date<FixedOffset>) = {
        if matches.is_present("range") {
            let range = matches.values_of("range").unwrap();
            let mut range: Vec<DateTime<FixedOffset>> = try!(range.map(|a| parsers::parse_datetime(a, ctime))
                    .collect());
            range.sort();
            (range[0].date(), range[1].date())
        } else {
            let ofst = ctime.offset().to_owned();
            (Date::from_utc(naive::date::MIN, ofst),
             Date::from_utc(naive::date::MAX, ofst))
        }
    };

    if matches.is_present("in") || matches.is_present("out") {
        let dir = if matches.is_present("in") {
            Direction::In
        } else {
            Direction::Out
        };

        let time = match matches.value_of("time") {
            Some(a) => {
                try!(parsers::parse_datetime(a, ctime)
                         .or_else(|_| parsers::parse_offset(a, ctime)))
            }
            None => ctime,
        };

        let memo = matches.value_of("memo").unwrap_or("");
        mark_time(dir, time, memo, &mut csv_file);

        println!("Clocked {:#} at {}", dir, time.format("%F %I:%M %P"));

    } else if matches.is_present("summary") || matches.is_present("range") {
        try!(print_summary(&csv_file, start_date, end_date, rounding));

    } else if matches.is_present("log") {
        try!(print_csv_entries(&csv_file));

    } else {
        let today = ctime.date();
        let weekday = today.weekday() as i64;
        let days_back = (7 - WEEKSTART + weekday) % 7;
        let start_date = today - Duration::days(days_back);
        try!(print_summary(&csv_file, start_date, today, rounding));
    }

    Ok(())
}


fn main() {
    match main0() {
        Ok(_) => {}
        Err(err) => {
            let _ = writeln!(&mut std::io::stderr(), "Error: {}", err);
        }
    };
}

#[cfg(test)]
mod tests {
    use super::*;
    use records::TimeEntryPair;
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
            assert!(slc[0].direction() == Direction::In);
            assert!(slc[1].direction() == Direction::Out);
        }
        // test ordering
        assert!(records[0].time() < records[1].time());
        assert!(records[1].time() < records[2].time());
        assert!(records[2].time() < records[3].time());
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
            assert!(tep.start().direction() == Direction::In);
            assert!(tep.end().direction() == Direction::Out);
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
            assert!(tep.start().direction() == Direction::In);
            assert!(tep.end().direction() == Direction::Out);
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
        assert_eq!(dr.duration(), 0.0);
        let dr = res.last().unwrap();
        assert_eq!(dr.date(), time.date() + day);
        assert_eq!(dr.duration(), 0.0);
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
        println!("\n{}", records[0].duration());
        assert!(records[0].duration() == 3600.0);
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
