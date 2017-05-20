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
