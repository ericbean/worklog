// Worklog is for recording your hours.
//

extern crate chrono;
extern crate csv;
extern crate getopts;
extern crate rustc_serialize;

mod error;
mod timeclock;
mod util;

use error::WorklogError;
use getopts::Options;
use std::env;
use std::error::Error;
use std::fs::File;
use std::fs::OpenOptions;
use std::path::PathBuf;
use timeclock::direction::Direction;


#[cfg(target_family = "unix")]
static CSV_FILE_NAME: &'static str = ".worklog.csv";
#[cfg(target_family = "windows")]
static CSV_FILE_NAME: &'static str = "worklog.csv";


fn print_full_summary(file: &File) -> Result<u8, WorklogError> {
    let csv_entries = try!(timeclock::read_timesheet(file));
    let entries = timeclock::pair_time_entries(csv_entries);
    let records = timeclock::collect_date_records(entries);

    let mut total_hours: f64 = 0.0;
    for rec in records {
        total_hours += rec.hours();
        println!("{}", rec);
    }

    println!("Total Hours: {:.2}", total_hours);
    Ok(0)
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

    } else if matches.opt_present("i") {
        let time = matches.opt_default("i", "now").unwrap();
        let time = try!(util::parse_multi_time_fmt(&time));
        timeclock::mark_time(Direction::In, time, String::new(), csv_file);
        println!("Clocked in at {}", time.to_rfc3339());

    } else if matches.opt_present("o") {
        let time = matches.opt_default("o", "now").unwrap();
        println!("time={}", time);
        let time = try!(util::parse_multi_time_fmt(&time));
        timeclock::mark_time(Direction::Out, time, String::new(), csv_file);
        println!("Clocked out at {}", time.to_rfc3339());

    } else {
        try!(print_full_summary(&csv_file));
    }

    Ok(0)
}


fn main() {
    match main0() {
        Ok(_) => {}
        Err(err) => println!("Whoops! {}", err.description()),
    };
}
