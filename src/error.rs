error_chain! {
    errors {
        InCompleteWire {
            display("wire data is incomplete"),
        }
        TooLongName {
            display("name is too long"),
        }
        TooLongLabel {
            display("label is too long"),
        }
        InvalidDecimalFormat {
            display("decimal format isn't valid"),
        }
        NoneTerminateLabel {
            display("none terminate label"),
        }
        DuplicatePeriod {
            display("period is duplicate"),
        }
        UnknownRRType {
            display("unknown rr type"),
        }
        InvalidLabelCharacter {
            display("invalid label character"),
        }
        BadCompressPointer {
            display("compress format isn't valid"),
        }
        InCompleteName {
            display("name isn't complete"),
        }
        RdataLenIsNotCorrect {
            display("length of rdata isn't correct"),
        }
        InvalidIPv4Address {
            display("invalid ipv4 address"),
        }
        ShortOfQuestion {
            display("no question is provided"),
        }
        InvalidLabelIndex {
            display("label index is invalid"),
        }
    }
}
