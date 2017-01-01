use chrono;
use getopts;
use std::env;
use std::error::Error;
use std::fmt;
use std::io;
use timeclock::error::TimeClockError;

#[derive(Debug)]
pub enum WorklogError {
    Env(env::VarError),
    Io(io::Error),
    Opts(getopts::Fail),
    CronoParse(chrono::ParseError),
    TimeClock(TimeClockError),
}


impl From<env::VarError> for WorklogError {
    fn from(err: env::VarError) -> WorklogError {
        WorklogError::Env(err)
    }
}


impl From<io::Error> for WorklogError {
    fn from(err: io::Error) -> WorklogError {
        WorklogError::Io(err)
    }
}


impl From<getopts::Fail> for WorklogError {
    fn from(err: getopts::Fail) -> WorklogError {
        WorklogError::Opts(err)
    }
}


impl From<chrono::ParseError> for WorklogError {
    fn from(err: chrono::ParseError) -> WorklogError {
        WorklogError::CronoParse(err)
    }
}


impl From<TimeClockError> for WorklogError {
    fn from(err: TimeClockError) -> WorklogError {
        WorklogError::TimeClock(err)
    }
}


impl Error for WorklogError {
    fn description(&self) -> &str {
        match *self {
            WorklogError::Env(ref err) => err.description(),
            WorklogError::Io(ref err) => err.description(),
            WorklogError::Opts(ref err) => err.description(),
            WorklogError::CronoParse(ref err) => err.description(),
            WorklogError::TimeClock(ref err) => err.description(),
        }
    }

    fn cause(&self) -> Option<&Error> {
        Some(match *self {
            WorklogError::Env(ref err) => err as &Error,
            WorklogError::Io(ref err) => err as &Error,
            WorklogError::Opts(ref err) => err as &Error,
            WorklogError::CronoParse(ref err) => err as &Error,
            WorklogError::TimeClock(ref err) => err as &Error,
        })
    }
}


impl fmt::Display for WorklogError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            WorklogError::Env(ref err) => fmt::Display::fmt(err, f),
            WorklogError::Io(ref err) => fmt::Display::fmt(err, f),
            WorklogError::Opts(ref err) => fmt::Display::fmt(err, f),
            WorklogError::CronoParse(ref err) => fmt::Display::fmt(err, f),
            WorklogError::TimeClock(ref err) => fmt::Display::fmt(err, f),
        }
    }
}
