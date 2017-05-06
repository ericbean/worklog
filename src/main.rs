// Worklog is for recording your hours.
//
extern crate chrono;
extern crate csv;
extern crate clap;
extern crate rustc_serialize;

mod error;
mod timeclock;
mod util;
mod parsers;

use chrono::*;
use clap::{Arg, ArgGroup, App};
use error::WorklogError;
use std::env;
use std::fs::OpenOptions;
use std::io::prelude::*;
use std::path::PathBuf;
use timeclock::Direction;
use timeclock::now;


#[cfg(target_family = "unix")]
static CSV_FILE_NAME: &'static str = ".worklog.csv";
#[cfg(target_family = "windows")]
static CSV_FILE_NAME: &'static str = "worklog.csv";

// default rounding mode
static ROUNDING_DEFAULT: &'static str = "+15m";

// TODO This belongs in a config file somewhere
const WEEKSTART: i64 = Weekday::Sat as i64;


fn print_csv_entries<R: Read>(file: R) -> Result<(), WorklogError> {
    let csv_entries = try!(timeclock::read_timesheet(file));
    for rec in csv_entries {
        println!("{}", rec);
    }
    Ok(())
}


fn print_full_summary<R: Read>(file: R,
                               rounding: util::Rounding)
                               -> Result<(), WorklogError> {
    let csv_entries = try!(timeclock::read_timesheet(file));
    let records = timeclock::collect_date_records(csv_entries);

    let mut total_hours: f64 = 0.0;
    for rec in records {
        let hours = util::round(rec.seconds(), rounding) / 3600.0;
        total_hours += hours;
        println!("{} {:.2} {}", rec.date().format("%F"), hours, rec.memo());
    }

    println!("Total Hours: {:.2}", total_hours);
    Ok(())
}


fn print_short_summary<R: Read>(file: R,
                                start_date: Date<FixedOffset>,
                                end_date: Date<FixedOffset>,
                                rounding: util::Rounding)
                                -> Result<(), WorklogError> {
    let csv_entries = try!(timeclock::read_timesheet(file));
    let records = timeclock::collect_date_records(csv_entries);

    let mut total_hours: f64 = 0.0;
    for rec in records {
        if start_date <= rec.date() && rec.date() <= end_date {
            let hours = util::round(rec.seconds(), rounding) / 3600.0;
            total_hours += hours;
            println!("{} {:.2} {}", rec.date().format("%F"), hours, rec.memo());
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
            .conflicts_with("log")
            .conflicts_with("inout"))
        .arg(Arg::from_usage("[range] --range <TIME> <TIME> 'range'")
            .conflicts_with("summary")
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

    if matches.is_present("in") || matches.is_present("out") {
        let dir = if matches.is_present("in") {
            Direction::In
        } else {
            Direction::Out
        };

        let time = match matches.value_of("time") {
            Some(a) => try!(parsers::parse_datetime(&a, now()).or(parsers::parse_offset(&a, now()))),
            None => now(),
        };

        let memo = matches.value_of("memo").unwrap_or("").to_owned();
        timeclock::mark_time(dir, time, memo, &mut csv_file);

        println!("Clocked {:#} at {}", dir, time.format("%F %I:%M %P"));

    } else if matches.is_present("summary") {
        try!(print_full_summary(&csv_file, rounding));

    } else if matches.is_present("log") {
        try!(print_csv_entries(&csv_file));

    } else if matches.is_present("range") {
        let time = now();
        let range = matches.values_of("range").unwrap();//.collect();
        let range: Vec<DateTime<FixedOffset>> =
            try!(range.map(|a| parsers::parse_datetime(a, time)).collect());
        try!(print_short_summary(&csv_file,
                                 range[0].date(),
                                 range[1].date(),
                                 rounding));

    } else {
        let today = now().date();
        let weekday = today.weekday() as i64;
        let days_back = (7 - WEEKSTART + weekday) % 7;
        let start_date = today - Duration::days(days_back);
        try!(print_short_summary(&csv_file, start_date, today, rounding));
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
