// Worklog is for recording your hours.
//

extern crate chrono;
extern crate csv;
extern crate getopts;
extern crate rustc_serialize;

mod error;
mod timeclock;
mod util;

use chrono::*;
use error::WorklogError;
use getopts::Options;
use std::env;
use std::error::Error;
use std::fs::OpenOptions;
use std::io::prelude::*;
use std::path::PathBuf;
use timeclock::direction::Direction;


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
    let entries = timeclock::pair_time_entries(csv_entries);
    let records = timeclock::collect_date_records(entries);

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
    let entries = timeclock::pair_time_entries(csv_entries);
    let records = timeclock::collect_date_records(entries);

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


fn print_usage(program: &str, opts: Options) {
    let brief = format!("Usage: {} [options]", program);
    print!("{}", opts.usage(&brief));
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
    return Ok(data_path);
}


fn main0() -> Result<i8, WorklogError> {
    let args: Vec<String> = env::args().collect();
    let program = args[0].clone();

    let mut opts = Options::new();
    opts.optflagopt("i", "in", "in time", "TIME");
    opts.optflagopt("o", "out", "out time", "TIME");
    opts.optopt("m", "memo", "the memo", "MEMO");
    opts.optflag("s", "summary", "show the full summary");
    opts.optflag("e", "entries", "show all entries");
    opts.optflag("h", "help", "print this help menu");

    let matches = try!(opts.parse(&args[1..]));

    let csv_path = try!(get_csv_path());

    let csv_file = try!(OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .open(csv_path));

    if matches.opt_present("h") {
        print_usage(&program, opts);

    } else if matches.opts_present(&["i".to_owned(), "o".to_owned()]) {
        // The ridiculous array above isn't my fault.

        // Reject --in and --out if present at the same time
        if matches.opt_present("i") && matches.opt_present("o") {
            println!("--in or --out, not both");
            return Ok(1);
        }

        let dir = match matches.opt_present("i") {
            true => Direction::In,
            false => Direction::Out,
        };

        let time = matches.opt_str("i")
            .or(matches.opt_str("o"))
            .or(Some("now".to_owned()))
            .unwrap();
        let time = try!(util::parse_multi_time_fmt(&time));

        let memo = matches.opt_str("m").or(Some("".to_owned())).unwrap();
        timeclock::mark_time(dir, time, memo, csv_file);

        println!("Clocked {:#} at {}", dir, time.format("%F %I:%M %P"));

    } else if matches.opt_present("s") {
        try!(print_full_summary(&csv_file));

    } else if matches.opt_present("e") {
        try!(print_csv_entries(&csv_file));

    } else {
        let days_back = (-WEEKSTART + 7) % 7;
        let start_date = util::now().date() - Duration::days(days_back);
        try!(print_short_summary(&csv_file, start_date));
    }

    Ok(0)
}


fn main() {
    match main0() {
        Ok(_) => {}
        Err(err) => println!("Whoops! {}", err.description()),
    };
}
