use rustc_serialize::{Decodable, Decoder};
use rustc_serialize::{Encodable, Encoder};
use std::fmt;


#[derive(Copy,Clone,Debug,PartialEq)]
pub enum Direction {
    In,
    Out,
}


impl Direction {
    pub fn reverse(&self) -> Direction {
        match self {
            &Direction::In => Direction::Out,
            &Direction::Out => Direction::In,
        }
    }
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

        match s.to_lowercase().as_ref() {
            "in" => return Ok(Direction::In),
            "out" => return Ok(Direction::Out),
            &_ => return Err(d.error("Field must be \"In\" or \"Out\"")),
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
        let width = f.width().or(Some(l)).unwrap();
        f.write_fmt(format_args!("{0:1$}", s, width))
    }
}


#[cfg(test)]
mod tests {
    // TODO figure out how to test encoding and decoding
}
