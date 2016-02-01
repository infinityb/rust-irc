#![deny(unused_must_use, unused_variables, unused_mut)]

#[macro_use] extern crate log;

pub use self::irccase::{
    OSCaseMapping,
    CaseMapping,
    IrcAsciiExt,
    OwnedIrcAsciiExt,
    AsciiCaseMapping,
    Rfc1459CaseMapping,
    StrictRfc1459CaseMapping,
};

#[cfg(feature="legacy")]
pub use self::parse::IrcMsg;

#[cfg(not(feature="legacy"))]
pub use self::parse::parse2::{IrcMsg, IrcMsgBuf};

#[cfg(test)] pub mod testinfra;
mod numerics;

mod slice;

/// Experimental message types
pub mod message_types;

/// Experimental utility code
pub mod util;

/// Experimental parsing code
pub mod parse;

pub mod identifier;

/// IRC case manipulation
mod irccase;

pub mod cap;
pub mod mtype2;
mod parse_helpers;
