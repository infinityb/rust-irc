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
    ParseErrorKind,
};

pub use self::mtype2::{server, client, FromIrcMsg};

#[cfg(test)] pub mod testinfra;

mod slice;

/// Experimental message group
pub mod message_group;

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

pub mod legacy;