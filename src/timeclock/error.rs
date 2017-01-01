use csv;
use std::error::Error;
use std::fmt;

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
