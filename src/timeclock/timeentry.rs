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
        write!(f, "({} {} {})", self.dir, self.time, self.memo)
    }
}
