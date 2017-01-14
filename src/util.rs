use chrono::*;
use error::WorklogError;
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

    parse_time(timestr).or_else(|_| parse_datetime(timestr))
}


#[cfg(test)]
mod tests {
    // huge pita to test these better. Someday though... Maybe.

    use super::*;
    use super::parse_datetime;
    use super::parse_time;

    #[test]
    fn parse_time_test() {
        assert!(parse_time("10:31").is_ok());
        assert!(parse_time("10:31AM").is_ok());
        assert!(parse_time("10:31 PM").is_ok());
        assert!(parse_time("10:31:12 PM").is_ok());
        assert!(parse_time("10:31:12.142134366").is_ok());
    }


    #[test]
    fn parse_datetime_test() {
        assert!(parse_datetime("2016-12-18 16:53:33.142134366").is_ok());
        assert!(parse_datetime("2016-12-18T16:53:33").is_ok());
        assert!(parse_datetime("2016-12-18T16:53").is_ok());
    }


    #[test]
    fn parse_multi_time_fmt_test() {
        // only testing "now" time
        assert!(parse_multi_time_fmt("now").is_ok());
    }
}