#![crate_name = "irc"]
#![crate_type = "dylib"]
#![license = "MIT/ASL2"]
#![feature(if_let, slicing_syntax, globs, phase, macro_rules)]

#![allow(dead_code, deprecated)]
#![deny(unused_must_use, warnings, unused_variables, unused_mut)]

#[cfg(test)] extern crate test;
#[phase(plugin, link)] extern crate log;
extern crate time;
extern crate serialize;

pub use self::message::IrcMessage;

pub use self::connection::IrcConnection;
pub use self::connection::IrcConnectionCommand;

pub use self::event::IrcEvent;

pub use self::watchers::{   
    RegisterError,
    RegisterErrorType,

    JoinResult,
    JoinSuccess,
    JoinError,

    WhoResult,
    WhoRecord,
    WhoSuccess,
    WhoError,

    BundlerManager,
    JoinBundlerTrigger,
    WhoBundlerTrigger,
};

pub use self::irccase::{
    IrcAsciiExt,
    OwnedIrcAsciiExt,
};

pub use self::state::{
    User,
    UserId,
    
    Channel,
    ChannelId,

    State,
    MessageEndpoint,
};

mod numerics;
mod connection;
mod message;
mod watchers;
mod core_plugins;

#[experimental = "Subject to all types of change"]
/// Experimental message types
pub mod message_types;

#[experimental = "Subject to being moved"]
/// Experimental utility code
pub mod util;

#[experimental = "Subject to being moved"]
/// Experimental parsing code
pub mod parse;

/// Event types
mod event;

/// IRC case manipulation
mod irccase;

/// IRC state tracker
mod state;