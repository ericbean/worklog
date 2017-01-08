use chrono::*;
use rustc_serialize::{Decodable, Decoder, Encodable, Encoder};
use std::fmt;
use timeclock::direction::Direction;


static DATETIMEFMT: &'static str = "%Y-%m-%dT%H:%M:%S%z";


fn fmt_datetime(time: DateTime<FixedOffset>) -> String {
    let dr = time.format(DATETIMEFMT);
    format!("{}", dr)
}


fn parse_datetime(s: &str) -> ParseResult<DateTime<FixedOffset>> {
    DateTime::parse_from_str(s, DATETIMEFMT)
}


#[derive(Clone, Debug)]
pub struct TimeEntry {
    pub dir: Direction,
    pub time: DateTime<FixedOffset>,
    pub memo: String,
}


impl TimeEntry {
    #[allow(dead_code)]
    pub fn new(dir: Direction,
               time: DateTime<FixedOffset>,
               memo: &str)
               -> Self {
        TimeEntry {
            dir: dir,
            time: time,
            memo: memo.to_owned(),
        }
    }
}


impl Encodable for TimeEntry {
    fn encode<S: Encoder>(&self, s: &mut S) -> Result<(), S::Error> {
        s.emit_struct("TimeEntry", 3, |s| {
            try!(s.emit_struct_field("dir", 0, |s| self.dir.encode(s)));
            try!(s.emit_struct_field("time", 1, |s| {
                return s.emit_str(fmt_datetime(self.time).as_ref());
            }));
            try!(s.emit_struct_field("memo", 2, |s| self.memo.encode(s)));
            Ok(())
        })
    }
}


impl Decodable for TimeEntry {
    fn decode<D: Decoder>(d: &mut D) -> Result<TimeEntry, D::Error> {
        d.read_struct("TimeEntry", 0, |d| {
            Ok(TimeEntry {
                dir: try!(d.read_struct_field("dir", 0, |d|
                        Decodable::decode(d))),
                time: {
                    let a_str: String = try!(d.read_struct_field("time", 0,
                            |d| d.read_str()));
                    let es = format!("Can't parse {}", a_str);
                    let dt = parse_datetime(&a_str);
                    match dt {
                        Ok(d) => d,
                        Err(_) => return Err(d.error(&es)),
                    }
                },
                memo: try!(d.read_struct_field("memo", 0, |d|
                        Decodable::decode(d))),
            })
        })
    }
}


impl fmt::Display for TimeEntry {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let time = self.time.format("%F %I:%M %P");
        f.write_fmt(format_args!("{:3} {} {}", self.dir, time, self.memo))
    }
}

#[cfg(test)]
mod tests {
    use csv;
    use std::io::Cursor;
    use super::*;
    use timeclock::direction::Direction;
    use timeclock::now;
    use timeclock::timeentry::{fmt_datetime, parse_datetime};

    #[test]
    fn timeentry_constructor_test() {
        let time = now();
        let te = TimeEntry::new(Direction::In, time, "Memo");
        assert_eq!(te.dir, Direction::In);
        assert_eq!(te.time, time);
        assert_eq!(te.memo, "Memo");
    }

    #[test]
    fn encode_timeentry_test() {
        let mut wtr = csv::Writer::from_memory();
        let timenow = now();
        let expected = format!("In,{},test memo\n", fmt_datetime(timenow));
        let _ = wtr.encode(TimeEntry {
            dir: Direction::In,
            time: timenow,
            memo: "test memo".to_owned(),
        });
        assert!(wtr.as_string() == expected);
    }

    #[test]
    fn decode_timeentry_test() {
        let timestr = "2017-01-05T14:04:16-0600";
        let expected_time = parse_datetime(timestr).unwrap();
        let memo = "test memo";
        let s = format!("In,{0:},{1:}\nOut,{0:},{1:}\n", timestr, memo);
        let vs = s.as_bytes();
        let buff = Cursor::new(vs);
        let mut rdr = csv::Reader::from_reader(buff).has_headers(false);
        let records =
            rdr.decode().collect::<csv::Result<Vec<TimeEntry>>>().unwrap();

        assert!(records[0].dir == Direction::In);
        assert!(records[1].dir == Direction::Out);
        assert!(records[0].time == expected_time);
        assert!(records[1].time == expected_time);
        assert!(records[0].memo == memo);
        assert!(records[1].memo == memo);
        assert!(records.len() == 2);
    }
}