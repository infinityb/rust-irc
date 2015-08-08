#![crate_name = "irc"]
#![crate_type = "dylib"]
#![feature(convert, hasher_write, slice_patterns)]

#![allow(stable_features)]
// hasher_write was stabilised in rust 1.4

#![deny(unused_must_use, unused_variables, unused_mut)]

#[macro_use] extern crate log;

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
    FrozenState,
    MessageEndpoint,
};

#[cfg(test)] pub mod testinfra;
mod numerics;
mod watchers;
mod core_plugins;

/// Experimental message types
pub mod message_types;

/// Experimental utility code
pub mod util;

/// Experimental parsing code
pub mod parse;

pub mod identifier;

pub mod stream;

/// Event types
mod event;

/// IRC case manipulation
mod irccase;

/// IRC state tracker
mod state;

/// Receive buffer
pub mod recv;
