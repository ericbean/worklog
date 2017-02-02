// Worklog is for recording your hours.
//

extern crate chrono;
extern crate csv;
extern crate clap;
extern crate rustc_serialize;

mod error;
mod timeclock;
mod util;

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

// TODO This belongs in a config file somewhere
const WEEKSTART: i64 = Weekday::Sat as i64;


fn print_csv_entries<R: Read>(file: R) -> Result<(), WorklogError> {
    let csv_entries = try!(timeclock::read_timesheet(file));
    for rec in csv_entries {
        println!("{}", rec);
    }
    Ok(())
}


fn print_full_summary<R: Read>(file: R) -> Result<(), WorklogError> {
    let csv_entries = try!(timeclock::read_timesheet(file));
    let records = timeclock::collect_date_records(csv_entries);

    let mut total_hours: f64 = 0.0;
    for rec in records {
        total_hours += rec.hours();
        println!("{}", rec);
    }

    println!("Total Hours: {:.2}", total_hours);
    Ok(())
}


fn print_short_summary<R: Read>(file: R,
                                since: Date<FixedOffset>)
                                -> Result<(), WorklogError> {
    let csv_entries = try!(timeclock::read_timesheet(file));
    let records = timeclock::collect_date_records(csv_entries);

    let mut total_hours: f64 = 0.0;
    for rec in records {
        if rec.date() >= since {
            total_hours += rec.hours();
            println!("{}", rec);
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
    let matches = App::new(env!("CARGO_PKG_NAME"))
        .version(env!("CARGO_PKG_VERSION"))
        .author(env!("CARGO_PKG_AUTHORS"))
        .about(env!("CARGO_PKG_DESCRIPTION"))
        .arg(Arg::from_usage("[in] -i, --in 'Record an In entry'"))
        .arg(Arg::from_usage("[out] -o, --out 'Record an Out entry'"))
        .arg(Arg::from_usage("[time] -t, --time <TIME> 'time'")
            .default_value("now")
            .requires("inout"))
        .arg(Arg::from_usage("[memo] -m, --memo <MEMO> 'Memo for the entry'")
            .default_value("")
            .requires("inout"))
        .arg(Arg::from_usage("[summary] -s, --summary 'Print a summary'")
            .conflicts_with("inout"))
        .arg(Arg::from_usage("[log] -l, --log 'Print the full log'")
            .conflicts_with("inout"))
        .group(ArgGroup::with_name("inout").args(&["in", "out"]))
        .get_matches();

    let csv_path = try!(get_csv_path());

    let mut csv_file = try!(OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .open(csv_path));

    if matches.is_present("in") || matches.is_present("out") {
        let dir = if matches.is_present("in") {
            Direction::In
        } else {
            Direction::Out
        };

        let time = matches.value_of("time").unwrap();
        let time = try!(util::parse_multi_time_fmt(&time));

        let memo = matches.value_of("memo").unwrap().to_owned();
        timeclock::mark_time(dir, time, memo, &mut csv_file);

        println!("Clocked {:#} at {}", dir, time.format("%F %I:%M %P"));

    } else if matches.is_present("summary") {
        try!(print_full_summary(&csv_file));

    } else if matches.is_present("log") {
        try!(print_csv_entries(&csv_file));

    } else {
        let days_back = (-WEEKSTART + 7) % 7;
        let start_date = now().date() - Duration::days(days_back);
        try!(print_short_summary(&csv_file, start_date));
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
