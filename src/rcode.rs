use std::fmt;

#[derive(Debug)]
pub struct Rcode(u8);

const R_NOERROR: u8 = 0; //< 0: No error (RFC1035)
const R_FORMERR: u8 = 1; // Format error (RFC1035)
const R_SERVFAIL: u8 = 2; // Server failure (RFC1035)
const R_NXDOMAIN: u8 = 3; // Name Error (RFC1035)
const R_NOTIMP: u8 = 4; // Not Implemented (RFC1035)
const R_REFUSED: u8 = 5; // Refused (RFC1035)
const R_YXDOMAIN: u8 = 6; // Name unexpectedly exists (RFC2136)
const R_YXRRSET: u8 = 7; // RRset unexpectedly exists (RFC2136)
const R_NXRRSET: u8 = 8; // RRset should exist but not (RFC2136)
const R_NOTAUTH: u8 = 9; // Server isn't authoritative (RFC2136)
const R_NOTZONE: u8 = 10; // Name is not within the zone (RFC2136)

impl Rcode {
    #[inline]
    pub fn no_error() -> Rcode {
        Rcode(R_NOERROR)
    }

    #[inline]
    pub fn fmt_error() -> Rcode {
        Rcode(R_FORMERR)
    }

    #[inline]
    pub fn serv_fail() -> Rcode {
        Rcode(R_SERVFAIL)
    }

    #[inline]
    pub fn nx_domain() -> Rcode {
        Rcode(R_NXDOMAIN)
    }

    #[inline]
    pub fn not_implement() -> Rcode {
        Rcode(R_NOTIMP)
    }

    #[inline]
    pub fn refuse() -> Rcode {
        Rcode(R_REFUSED)
    }

    #[inline]
    pub fn name_should_not_exist() -> Rcode {
        Rcode(R_YXDOMAIN)
    }

    #[inline]
    pub fn rrset_should_not_exist() -> Rcode {
        Rcode(R_YXRRSET)
    }

    #[inline]
    pub fn rrset_should_exist() -> Rcode {
        Rcode(R_NXRRSET)
    }

    #[inline]
    pub fn not_auth() -> Rcode {
        Rcode(R_NOTAUTH)
    }

    #[inline]
    pub fn name_not_in_zone() -> Rcode {
        Rcode(R_NOTZONE)
    }

    #[inline]
    pub fn is_no_error(&self) -> bool {
        self.0 == R_NOERROR
    }
}


impl fmt::Display for Rcode {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let msg = match self.0 {
            R_NOERROR => "NOERROR",
            R_FORMERR => "FORMERR",
            R_SERVFAIL => "SERVFAIL",
            R_NXDOMAIN => "NXDOMAIN",
            R_NOTIMP => "NOTIMP",
            R_REFUSED => "REFUSED",
            R_YXDOMAIN => "YXDOMAIN",
            R_YXRRSET => "YXRRSET",
            R_NXRRSET => "NXRRSET",
            R_NOTAUTH => "NOTAUTH",
            R_NOTZONE => "NOTZONE",
            _ => "UNKOWN",
        };
        write!(f, "{}", msg)
    }
}
