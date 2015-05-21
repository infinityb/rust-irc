#![crate_type = "dylib"]
#![feature(core, collections, convert, hash, slice_patterns, into_cow)]

#![deny(unused_must_use, warnings, unused_variables, unused_mut)]

#[macro_use] extern crate log;

pub use self::event::{
    IrcEvent,
    IrcSender,
    Plugin,
    PluginManager,
};

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

pub use stream::{
    IrcConnectorExtension,
    IrcConnectionBuilder,
    IrcConnector,
};

pub use self::irccase::{
    IrcAsciiExt,
    OwnedIrcAsciiExt,
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
pub mod irccase;

/// Receive buffer
pub mod recv;