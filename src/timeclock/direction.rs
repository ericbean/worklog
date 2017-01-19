use rustc_serialize::{Decodable, Decoder};
use rustc_serialize::{Encodable, Encoder};
use std::fmt;


#[derive(Copy,Clone,Debug,PartialEq)]
pub enum Direction {
    In,
    Out,
}


impl Encodable for Direction {
    fn encode<S: Encoder>(&self, s: &mut S) -> Result<(), S::Error> {
        s.emit_enum("Direction", |s| {
            match *self {
                Direction::In => s.emit_enum_variant("In", 0, 0, |_| Ok(())),
                Direction::Out => s.emit_enum_variant("Out", 0, 0, |_| Ok(())),
            }
        })
    }
}


impl Decodable for Direction {
    fn decode<D: Decoder>(d: &mut D) -> Result<Direction, D::Error> {
        let s = match d.read_str() {
            Ok(s) => s,
            Err(e) => return Err(e),
        };

        match s.to_lowercase().trim().as_ref() {
            "in" => Ok(Direction::In),
            "out" => Ok(Direction::Out),
            &_ => Err(d.error("Field must be \"In\" or \"Out\"")),
        }
    }
}


impl fmt::Display for Direction {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let (s, l) = match (*self, f.alternate()) {
            (Direction::In, false) => ("In", 2),
            (Direction::Out, false) => ("Out", 3),
            (Direction::In, true) => ("in", 2),
            (Direction::Out, true) => ("out", 3),
        };
        let width = f.width().or_else(|| Some(l)).unwrap();
        f.write_fmt(format_args!("{0:1$}", s, width))
    }
}


#[cfg(test)]
mod tests {
    use csv;
    use std::io::Cursor;
    use super::*;

    #[test]
    fn encode_direction_test() {
        let mut wtr = csv::Writer::from_memory();
        let _ = wtr.encode(Direction::In);
        let _ = wtr.encode(Direction::Out);
        assert!(wtr.as_string() == "In\nOut\n");
    }

    #[test]
    fn decode_direction_test() {
        let s = "In\nOut\nin\nout\n In\nOut ";
        let vs = s.as_bytes();
        let buff = Cursor::new(vs);
        let mut rdr = csv::Reader::from_reader(buff).has_headers(false);
        let records =
            rdr.decode().collect::<csv::Result<Vec<Direction>>>().unwrap();

        println!("\nlen={}", records.len());
        for record in records.clone() {
            println!("{}", record);
        }

        for slc in records.chunks(2) {
            assert!(slc[0] == Direction::In);
            assert!(slc[1] == Direction::Out);
        }
        assert!(records.len() == 6);
    }

    #[test]
    #[should_panic(expected = "called `Result::unwrap()` on an `Err` value")]
    fn decode_direction_fail_test() {
        let s = "In\nDEADBEEF\nin\nout\n In\nOut ";
        let vs = s.as_bytes();
        let buff = Cursor::new(vs);
        let mut rdr = csv::Reader::from_reader(buff).has_headers(false);
        let _ = rdr.decode().collect::<csv::Result<Vec<Direction>>>().unwrap();
    }
}
