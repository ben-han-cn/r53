use std::fmt;

#[derive(Debug)]
pub enum Error {
    InCompleteWire,

    TooLongName,
    UnknownRRType,
    InvalidLabelCharacter,
    BadCompressPointer,
    InCompleteName,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let msg = match *self {
            Error::InCompleteWire => "wire format is incomplete",
            Error::UnknownRRType => "rr type is unknown",
            Error::TooLongName => "name is too long",
            Error::InvalidLabelCharacter => "character isn't valid",
            Error::BadCompressPointer => "compress pointer in name isn't valid",
            Error::InCompleteName => "name isn't completed",
        };

        f.write_fmt(format_args!("buffer error ({})", msg))
    }
}
