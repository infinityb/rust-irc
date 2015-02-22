#![crate_name = "irc"]
#![crate_type = "dylib"]
#![feature(core, std_misc, collections, hash, std_misc)]

#![allow(missing_copy_implementations)]
#![deny(unused_must_use, warnings, unused_variables, unused_mut)]

#[cfg(test)] extern crate test;
#[macro_use] extern crate log;

// pub use self::message::IrcMessage;
pub use self::connection::{IrcConnectionBuf, IrcConnection};
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

#[cfg(test)] pub mod testinfra;
mod numerics;
mod connection;
mod watchers;
mod core_plugins;

#[unstable(reason="Subject to all types of change")]
/// Experimental message types
pub mod message_types;

#[unstable(reason="Subject to being moved")]
/// Experimental utility code
pub mod util;

#[unstable(reason="Subject to being moved")]
/// Experimental parsing code
pub mod parse;

#[unstable(reason="Subject to all types of change")]
pub mod identifier;

#[unstable(reason="Subject to all types of change")]
pub mod stream;

/// Event types
mod event;

/// IRC case manipulation
mod irccase;

/// IRC state tracker
mod state;
