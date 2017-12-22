use std::fmt;

#[derive(Debug)]
pub enum Error {
    ReadOutOfRange,
    WriteOutOfRange,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let msg = match *self {
            Error::ReadOutOfRange => "Input buffer has no enough bytes to read",
            Error::WriteOutOfRange => "Write buffer has no enough bytes to write",
        };

        f.write_fmt(format_args!("buffer error ({})", msg))
    }
}
