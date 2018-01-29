#![recursion_limit = "1024"]

use parsers::{ParseError};
use csv::Error as CsvError;
use std::io::Error as IoError;

error_chain! {
    types {
        Error, ErrorKind, ResultExt, Result;
    }

    links {}

    foreign_links {
        Csv(CsvError);
        IoError(IoError);
        ParseError(ParseError);
    }

    errors {}
}