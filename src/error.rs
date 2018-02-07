use std::fmt;

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum Error {
    InCompleteWire,
    TooLongName,
    TooLongLabel,
    InvalidDecimalFormat,
    NoneTerminateLabel,
    DuplicatePeriod,
    UnknownRRType,
    InvalidLabelCharacter,
    BadCompressPointer,
    InCompleteName,
    RdataLenIsNotCorrect,
    InvalidIPv4Address,
    ShortOfQuestion,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let msg = match *self {
            Error::InCompleteWire => "wire format is incomplete",
            Error::UnknownRRType => "rr type is unknown",
            Error::TooLongName => "name is too long",
            Error::TooLongLabel => "label is too long",
            Error::InvalidDecimalFormat => "escaped decimal isn't valid",
            Error::NoneTerminateLabel => "non terminating empty label",
            Error::DuplicatePeriod => "duplicate period",
            Error::InvalidLabelCharacter => "character isn't valid",
            Error::BadCompressPointer => "compress pointer in name isn't valid",
            Error::InCompleteName => "name isn't completed",
            Error::RdataLenIsNotCorrect => "rdata len isn't correct",
            Error::InvalidIPv4Address => "ipv4 address isn't valid",
            Error::ShortOfQuestion => "message short of question",
        };

        f.write_str(msg)
    }
}
