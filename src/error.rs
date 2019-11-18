use failure::Fail;

#[derive(Debug, Fail)]
pub enum DNSError {
    #[fail(display = "wire data is incomplete")]
    InCompleteWire,

    #[fail(display = "name is too long")]
    TooLongName,

    #[fail(display = "label is too long")]
    TooLongLabel,

    #[fail(display = "decimal format isn't valid")]
    InvalidDecimalFormat,

    #[fail(display = "none terminate label")]
    NoneTerminateLabel,

    #[fail(display = "period is duplicate")]
    DuplicatePeriod,

    #[fail(display = "unknown rr type {}", _0)]
    UnknownRRType(u16),

    #[fail(display = "invalid label character")]
    InvalidLabelCharacter,

    #[fail(display = "compress format isn't valid")]
    BadCompressPointer,

    #[fail(display = "name isn't complete")]
    InCompleteName,

    #[fail(display = "length of rdata isn't correct")]
    RdataLenIsNotCorrect,

    #[fail(display = "invalid ipv4 address")]
    InvalidIPv4Address,

    #[fail(display = "invalid ipv6 address")]
    InvalidIPv6Address,

    #[fail(display = "no question is provided")]
    ShortOfQuestion,

    #[fail(display = "label index is invalid")]
    InvalidLabelIndex,

    #[fail(display = "string isn't valid rrset")]
    InvalidRRsetString,

    #[fail(display = "string isn't valid ttl")]
    InvalidTtlString,

    #[fail(display = "string isn't valid class")]
    InvalidClassString,

    #[fail(display = "rrtype isn't support yet")]
    RRTypeIsNotSupport,

    #[fail(display = "rdata field {} with type {} isn't valid: {}", _1, _0, _2)]
    InvalidRdataString(&'static str, &'static str, String),

    #[fail(
        display = "label seqences in concat_all, the last is absolute and others are not absolute"
    )]
    InvalidLabelSequnceConcatParam,
}
