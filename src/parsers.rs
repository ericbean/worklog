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
    Hour,
    Minute,
    Second,
    Nanosecond,
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
            ParseError::Hour => "The specified hours is invalid",
            ParseError::Minute => "The specified minutes is invalid",
            ParseError::Second => "The specified seconds is invalid",
            ParseError::Nanosecond => "The specified nanoseconds is invalid",
        }
    }

    fn cause(&self) -> Option<&Error> {
        match *self {
            ParseError::PE(ref err) => Some(err as &Error),
            _ => None,
        }
    }
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            ParseError::PE(ref err) => fmt::Display::fmt(err, f),
            _ => fmt::Display::fmt(self.description(), f),
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

#[allow(dead_code)] // This function may go away if I don't find a use for it soon
pub fn parse_time(input: &str,
                  time: DateTime<FixedOffset>)
                  -> Result<DateTime<FixedOffset>, ParseError> {
    let (hour, minute, second) = try!(grammar::time(input));
    let nanosecond = second.fract() * 1_000_000_000.0;
    let time = try!(time.with_hour(hour).ok_or(ParseError::Hour));
    let time = try!(time.with_minute(minute).ok_or(ParseError::Minute));
    let time = try!(time.with_second(second as u32).ok_or(ParseError::Second));
    let time = try!(time.with_nanosecond(nanosecond as u32)
        .ok_or(ParseError::Nanosecond));
    Ok(time)
}


pub fn parse_datetime(input: &str,
                      time: DateTime<FixedOffset>)
                      -> Result<DateTime<FixedOffset>, ParseError> {
    let (year, month, day, hour, minute, second, tz) =
        try!(grammar::datetime(input));

    let year = year.unwrap_or(time.year());
    let month = month.unwrap_or(time.month());
    let day = day.unwrap_or(time.day());

    // set timezone
    let tz = tz.unwrap_or(time.offset().utc_minus_local());
    let tzo = FixedOffset::west(tz);
    let time = tzo.ymd(year, month, day);

    // set the time
    let time = time.and_hms(0, 0, 0);
    let time = try!(time.with_hour(hour).ok_or(ParseError::Hour));
    let time = try!(time.with_minute(minute).ok_or(ParseError::Minute));
    let time = try!(time.with_second(second as u32).ok_or(ParseError::Second));
    let nanosecond = second.fract() * 1_000_000_000.0;
    let time = try!(time.with_nanosecond(nanosecond as u32)
        .ok_or(ParseError::Nanosecond));
    Ok(time)
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

        assert_eq!(parse_rounding("-30m").unwrap(), Rounding::Down(1800.0));
        assert_eq!(parse_rounding("_30m").unwrap(), Rounding::Down(1800.0));
        assert_eq!(parse_rounding("D30m").unwrap(), Rounding::Down(1800.0));
        assert_eq!(parse_rounding("d7.5m").unwrap(), Rounding::Down(450.0));

        assert_eq!(parse_rounding("=1h").unwrap(), Rounding::Half(3600.0));
        assert_eq!(parse_rounding("H1h").unwrap(), Rounding::Half(3600.0));
        assert_eq!(parse_rounding("h7.5m").unwrap(), Rounding::Half(450.0));

        assert_eq!(parse_rounding("+30S").unwrap(), Rounding::Up(30.0));
        assert_eq!(parse_rounding("D30s").unwrap(), Rounding::Down(30.0));
        assert_eq!(parse_rounding("h30s").unwrap(), Rounding::Half(30.0));
        assert_eq!(parse_rounding("H1d").unwrap(), Rounding::Half(86400.0));
        assert_eq!(parse_rounding("e1d").unwrap(), Rounding::Half(86400.0));
        assert_eq!(parse_rounding("7.5").unwrap(), Rounding::Half(450.0));
    }

    #[test]
    fn offset_test() {
        assert_eq!(grammar::offset("+2.3s").unwrap(), 2.3);
        assert_eq!(grammar::offset("-1.55h").unwrap(), -5580.0);
        assert_eq!(grammar::offset("+2:22").unwrap(), 8520.0);
        assert_eq!(grammar::offset("_2:22").unwrap(), -8520.0);
    }

    #[test]
    fn parse_offset_test() {
        let ctime = now();
        let n = ctime.checked_add_signed(Duration::seconds(5580)).unwrap();
        assert_eq!(parse_offset("+1.55h", ctime).unwrap(), n);
    }

    #[test]
    fn time_test() {
        assert_eq!(grammar::time("09:22:32").unwrap(), (9, 22, 32.0));
        assert_eq!(grammar::time("9:22:32").unwrap(), (9, 22, 32.0));
        assert_eq!(grammar::time("9:22:32.055").unwrap(), (9, 22, 32.055));
        assert_eq!(grammar::time("9:22:32 Am").unwrap(), (9, 22, 32.0));
        assert_eq!(grammar::time("9:22:32.055Pm").unwrap(), (21, 22, 32.055));
        assert_eq!(grammar::time("9:22 Pm").unwrap(), (21, 22, 0.0));
        // Might want to fail on these someday
        assert_eq!(grammar::time("9:22:32. Am").unwrap(), (9, 22, 32.0));
        assert_eq!(grammar::time("9:22: Pm").unwrap(), (21, 22, 0.0));
        assert_eq!(grammar::time("99:22:32").unwrap(), (99, 22, 32.0));
    }

    #[test]
    fn parse_time_test() {
        let ctime: DateTime<FixedOffset> =
            "2017-04-30T15:55:31.961764802-05:00".parse().unwrap();
        assert_eq!(parse_time("9:22:31.9617 Pm", ctime).unwrap().to_rfc3339(),
                   "2017-04-30T21:22:31.961700416-05:00");
        assert_eq!(parse_time("9:22", ctime).unwrap().to_rfc3339(),
                   "2017-04-30T09:22:00-05:00");
    }

    #[test]
    fn datetime_test() {
        assert_eq!(grammar::datetime("4/2 9:22 Pm").unwrap(),
                   (None, Some(4), Some(2), 21, 22, 0.0, None));
        assert_eq!(grammar::datetime("2017-4/2 9:22 Pm").unwrap(),
                   (Some(2017), Some(4), Some(2), 21, 22, 0.0, None));
        assert_eq!(grammar::datetime("2017-4/2").unwrap(),
                   (Some(2017), Some(4), Some(2), 0, 0, 0.0, None));
        assert_eq!(grammar::datetime("9:22 Pm").unwrap(),
                   (None, None, None, 21, 22, 0.0, None));
        assert_eq!(grammar::datetime("12:24 Pm").unwrap(),
                   (None, None, None, 12, 24, 0.0, None));
    }

    #[test]
    fn parse_datetime_test() {
        let ctime: DateTime<FixedOffset> =
            "2017-04-30T15:55:31.961764802-05:00".parse().unwrap();
        assert_eq!(parse_datetime("9:22:31.9617 Pm", ctime)
                       .unwrap()
                       .to_rfc3339(),
                   "2017-04-30T21:22:31.961700416-05:00");
        assert_eq!(parse_datetime("9:22", ctime).unwrap().to_rfc3339(),
                   "2017-04-30T09:22:00-05:00");
        assert_eq!(parse_datetime("2017-4-30 9:22", ctime)
                       .unwrap()
                       .to_rfc3339(),
                   "2017-04-30T09:22:00-05:00");
        assert_eq!(parse_datetime("2017-4-30", ctime).unwrap().to_rfc3339(),
                   "2017-04-30T00:00:00-05:00");
        // 12pm == midnight bug
        assert_eq!(parse_datetime("12:24pm", ctime).unwrap().to_rfc3339(),
                   "2017-04-30T12:24:00-05:00");
        assert_eq!(parse_datetime("1:24pm", ctime).unwrap().to_rfc3339(),
                   "2017-04-30T13:24:00-05:00");
        assert_eq!(parse_datetime("1:24am", ctime).unwrap().to_rfc3339(),
                   "2017-04-30T01:24:00-05:00");
    }

    #[test]
    #[should_panic(expected = "ParseError")]
    fn parse_datetime_seperator_test() {
        let ctime: DateTime<FixedOffset> =
            "2017-04-30T15:55:31.961764802-05:00".parse().unwrap();
        assert_eq!(parse_datetime("2017-4-309:22", ctime)
                       .unwrap()
                       .to_rfc3339(),
                   "2017-04-30T09:22:00-05:00");
    }
}
