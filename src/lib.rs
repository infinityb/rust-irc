#![deny(unused_must_use, unused_variables, unused_mut)]

#[macro_use] extern crate log;
extern crate unicase;

pub use self::irccase::{
    OSCaseMapping,
    CaseMapping,
    IrcAsciiExt,
    OwnedIrcAsciiExt,
    AsciiCaseMapping,
    Rfc1459CaseMapping,
    StrictRfc1459CaseMapping,
};

pub use self::parse::{
    IrcMsg,
    IrcMsgBuf,
    ParseError,
};

pub use self::mtype2::{server, client};

#[cfg(test)] pub mod testinfra;

mod slice;

/// Experimental message types
mod message_types;

/// Experimental utility code
mod util;

/// Experimental parsing code
mod parse;

pub mod identifier;

/// IRC case manipulation
mod irccase;

mod mtype2;
mod parse_helpers;

#[cfg(feature = "unstable")]
pub mod cap;
