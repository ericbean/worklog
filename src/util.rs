use chrono::*;
use error::WorklogError;
use parsers::parse_offset;
use timeclock::now;

/// Helper fn for `parse_multi_time_fmt`
/// Parse time from various formats. Returns the current date combined with
/// the parsed time.
fn parse_time(timestr: &str) -> Result<DateTime<FixedOffset>, WorklogError> {
    let ctime = now();
    let cdate = ctime.date();
    let res = try!(NaiveTime::parse_from_str(&timestr, "%H:%M:%S%.f")
        .or(NaiveTime::parse_from_str(&timestr, "%I:%M:%S%.f %p"))
        .or(NaiveTime::parse_from_str(&timestr, "%I:%M %p"))
        .or(NaiveTime::parse_from_str(&timestr, "%H:%M")));

    // unwrap() here isn't ideal, but and_time() can panic anyway
    Ok(cdate.and_time(res).unwrap())
}

/// Helper fn for `parse_multi_time_fmt`
/// Parse a datetime from various formats.
fn parse_datetime(timestr: &str)
                  -> Result<DateTime<FixedOffset>, WorklogError> {
    let ctime = now();
    let offset = ctime.offset();
    // reduce the number of formats to check
    let timestr = timestr.replace("T", " ").replace("/", "-");
    let res = try!(offset.datetime_from_str(&timestr, "%Y-%m-%d %H:%M:%S%.f")
        .or(offset.datetime_from_str(&timestr, "%Y-%m-%d %H:%M:%S%.f"))
        // 2016/12/18 10:31 PM not parsable?
        // .or(offset.datetime_from_str(&timestr, "%Y-%m-%d %I:%M %p"))
        .or(offset.datetime_from_str(&timestr, "%Y-%m-%d %H:%M")));

    Ok(res)

}

/// Try to parse dates and times in as many formats as reasonable
pub fn parse_multi_time_fmt(timestr: &str)
                            -> Result<DateTime<FixedOffset>, WorklogError> {
    // We can pass "now" in and get the current time.
    // Simplifies dealing with -i and -o options
    if timestr == "now" {
        return Ok(now());
    }

    parse_offset(&timestr, now())
        .or(parse_time(timestr))
        .or(parse_datetime(timestr))
}

/// Rounding modes for round()
#[derive(Copy,Clone,Debug,PartialEq)]
pub enum Rounding {
    Up(f32),
    Down(f32),
    Half(f32),
    None,
}

/// Round f64 with Rounding mode
pub fn round(seconds: f64, rounding: Rounding) -> f64 {
    let res = match rounding {
        Rounding::Up(r) => (seconds / r as f64).ceil() * r as f64,
        Rounding::Down(r) => (seconds / r as f64).floor() * r as f64,
        Rounding::Half(r) => (seconds / r as f64).round() * r as f64,
        Rounding::None => seconds,
    };
    // don't return NaN
    if res.is_nan() { seconds } else { res }
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::parse_datetime;
    use super::parse_time;

    #[test]
    fn util_parse_time_test() {
        assert!(parse_time("10:31").is_ok());
        assert!(parse_time("10:31AM").is_ok());
        assert!(parse_time("10:31 PM").is_ok());
        assert!(parse_time("10:31:12 PM").is_ok());
        assert!(parse_time("10:31:12.142134366").is_ok());
    }

    #[test]
    fn util_parse_datetime_test() {
        assert!(parse_datetime("2016-12-18 16:53:33.142134366").is_ok());
        assert!(parse_datetime("2016-12-18T16:53:33").is_ok());
        assert!(parse_datetime("2016-12-18T16:53").is_ok());
    }

    #[test]
    fn util_parse_multi_time_fmt_test() {
        assert!(parse_multi_time_fmt("now").is_ok());
        assert!(parse_multi_time_fmt("-1:55").is_ok());
        assert!(parse_multi_time_fmt("dfggfh").is_err());
    }

    #[test]
    fn round_up_test() {
        const TIME_ACTUAL: f64 = 38160.12345; // ~10:36am in seconds
        const TIME_UP: f64 = 38700.0; // 10:45am in seconds
        const TIME_DOWN: f64 = 37800.0; // 10:30am in seconds

        // round up
        let res = round(TIME_ACTUAL, Rounding::Up(900.0));
        assert_eq!(res, TIME_UP);
        let res = round(TIME_UP, Rounding::Up(900.0));
        assert_eq!(res, TIME_UP);
        let res = round(TIME_UP - 1.0, Rounding::Up(900.0));
        assert_eq!(res, TIME_UP);

        // round down
        let res = round(TIME_ACTUAL, Rounding::Down(900.0));
        assert_eq!(res, TIME_DOWN);
        let res = round(TIME_DOWN, Rounding::Down(900.0));
        assert_eq!(res, TIME_DOWN);
        let res = round(TIME_DOWN + 1.0, Rounding::Down(900.0));
        assert_eq!(res, TIME_DOWN);

        // round half
        let res = round(TIME_ACTUAL, Rounding::Half(900.0));
        assert_eq!(res, TIME_DOWN);
        let res = round(TIME_DOWN, Rounding::Half(900.0));
        assert_eq!(res, TIME_DOWN);

        // round to 1/4 sec
        let res = round(TIME_ACTUAL, Rounding::Up(0.25));
        assert_eq!(res, 38160.25);

        // round with zero
        let res = round(TIME_ACTUAL, Rounding::Up(0.0));
        assert_eq!(res, TIME_ACTUAL);

        // round None
        let res = round(TIME_ACTUAL, Rounding::None);
        assert_eq!(res, TIME_ACTUAL);

    }
}
