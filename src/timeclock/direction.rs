use std::fmt;

#[derive(Copy,Clone,Debug,PartialEq,Serialize,Deserialize)]
pub enum Direction {
    In,
    Out,
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
    use super::*;
    use csv;
    use std::io::Cursor;

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
        let records = rdr.decode()
            .collect::<csv::Result<Vec<Direction>>>()
            .unwrap();

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
        let _ = rdr.decode()
            .collect::<csv::Result<Vec<Direction>>>()
            .unwrap();
    }
}
