use chrono;
use csv;
use parsers::ParseError;
use std::env;
use std::error::Error;
use std::fmt;
use std::io;

#[derive(Debug)]
pub enum WorklogError {
    Env(env::VarError),
    Io(io::Error),
    CronoParse(chrono::ParseError),
    TimeClock(TimeClockError),
    ParseError(ParseError),
}

impl From<ParseError> for WorklogError {
    fn from(err: ParseError) -> WorklogError {
        WorklogError::ParseError(err)
    }
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
            WorklogError::CronoParse(ref err) => err.description(),
            WorklogError::TimeClock(ref err) => err.description(),
            WorklogError::ParseError(ref err) => err.description(),
        }
    }

    fn cause(&self) -> Option<&Error> {
        match *self {
            WorklogError::Env(ref err) => Some(err as &Error),
            WorklogError::Io(ref err) => Some(err as &Error),
            WorklogError::CronoParse(ref err) => Some(err as &Error),
            WorklogError::TimeClock(ref err) => Some(err as &Error),
            WorklogError::ParseError(ref err) => Some(err as &Error),
        }
    }
}


impl fmt::Display for WorklogError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            WorklogError::Env(ref err) => fmt::Display::fmt(err, f),
            WorklogError::Io(ref err) => fmt::Display::fmt(err, f),
            WorklogError::CronoParse(ref err) => fmt::Display::fmt(err, f),
            WorklogError::TimeClock(ref err) => fmt::Display::fmt(err, f),
            WorklogError::ParseError(ref err) => fmt::Display::fmt(err, f),
        }
    }
}

#[derive(Debug)]
pub enum TimeClockError {
    Csv(csv::Error),
}


impl From<csv::Error> for TimeClockError {
    fn from(err: csv::Error) -> TimeClockError {
        TimeClockError::Csv(err)
    }
}


impl Error for TimeClockError {
    fn description(&self) -> &str {
        match *self {
            TimeClockError::Csv(ref err) => err.description(),
        }
    }

    fn cause(&self) -> Option<&Error> {
        Some(match *self {
                 TimeClockError::Csv(ref err) => err as &Error,
             })
    }
}


impl fmt::Display for TimeClockError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            TimeClockError::Csv(ref err) => fmt::Display::fmt(err, f),
        }
    }
}
