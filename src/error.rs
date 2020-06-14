use std::io;
use std::result;

use quick_error::quick_error;

pub type Result<T> = result::Result<T, Error>;

quick_error! {
    #[derive(Debug)]
    pub enum Error {
        Io(err: io::Error, source: String) {
            display("I/O error with {}: {}", source, err)
            context(source: &'a String, err: io::Error)
                -> (err, source.to_string())
        }
        Parse(message: String) {
            display("Parse error: {}", message)
        }
    }
}
