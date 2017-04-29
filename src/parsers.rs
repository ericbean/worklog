mod grammar {
    include!(concat!(env!("OUT_DIR"), "/grammar.rs"));
}

use chrono::Duration;
use chrono::prelude::*;
use std::error::Error;
use std::fmt;
use util::Rounding;

#[derive(Debug)]
pub enum ParseError {
    PE(grammar::ParseError),
    Overflow,
}

impl From<grammar::ParseError> for ParseError {
    fn from(err: grammar::ParseError) -> ParseError {
        ParseError::PE(err)
    }
}

impl Error for ParseError {
    fn description(&self) -> &str {
        match *self {
            ParseError::PE(ref err) => err.description(),
            ParseError::Overflow => "Invalid input",
        }
    }

    fn cause(&self) -> Option<&Error> {
        match *self {
            ParseError::PE(ref err) => Some(err as &Error),
            ParseError::Overflow => None,
        }
    }
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            ParseError::PE(ref err) => fmt::Display::fmt(err, f),
            ParseError::Overflow => fmt::Display::fmt(self.description(), f),
        }
    }
}

pub fn parse_rounding(fmt: &str) -> Result<Rounding, ParseError> {
    let res = try!(grammar::rounding(fmt));
    return Ok(res);
}

pub fn parse_offset(offset: &str,
                    time: DateTime<FixedOffset>)
                    -> Result<DateTime<FixedOffset>, ParseError> {
    let offset = try!(grammar::offset(offset));
    let offset = Duration::seconds(offset as i64);
    time.checked_add_signed(offset).ok_or(ParseError::Overflow)

}

#[cfg(test)]
mod tests {
    use super::*;
    use timeclock::now;

    #[test]
    fn parse_rounding_test() {
        assert_eq!(parse_rounding("+15m").unwrap(), Rounding::Up(900.0));
        assert_eq!(parse_rounding("U15m").unwrap(), Rounding::Up(900.0));
        assert_eq!(parse_rounding("u15m").unwrap(), Rounding::Up(900.0));
        assert_eq!(parse_rounding("7.5").unwrap(), Rounding::Up(450.0));

        assert_eq!(parse_rounding("-30m").unwrap(), Rounding::Down(1800.0));
        assert_eq!(parse_rounding("D30m").unwrap(), Rounding::Down(1800.0));
        assert_eq!(parse_rounding("d7.5m").unwrap(), Rounding::Down(450.0));

        assert_eq!(parse_rounding("=1h").unwrap(), Rounding::Half(3600.0));
        assert_eq!(parse_rounding("H1h").unwrap(), Rounding::Half(3600.0));
        assert_eq!(parse_rounding("h7.5m").unwrap(), Rounding::Half(450.0));

        assert_eq!(parse_rounding("+30S").unwrap(), Rounding::Up(30.0));
        assert_eq!(parse_rounding("D30s").unwrap(), Rounding::Down(30.0));
        assert_eq!(parse_rounding("h30s").unwrap(), Rounding::Half(30.0));
        assert_eq!(parse_rounding("H1d").unwrap(), Rounding::Half(86400.0));
    }

    #[test]
    fn offset_test() {
        assert_eq!(grammar::offset("2.3s").unwrap(), 2.3);
        assert_eq!(grammar::offset("+2.3s").unwrap(), 2.3);
        assert_eq!(grammar::offset("1.55h").unwrap(), 5580.0);
        assert_eq!(grammar::offset("2:22").unwrap(), 8520.0);
        assert_eq!(grammar::offset("+2:22").unwrap(), 8520.0);
        assert_eq!(grammar::offset("-2:22").unwrap(), -8520.0);
    }

    #[test]
    fn parse_offset_test() {
        let ctime = now();
        let n = ctime.checked_add_signed(Duration::seconds(5580)).unwrap();
        assert_eq!(parse_offset("1.55h", ctime).unwrap(), n);
    }
}
