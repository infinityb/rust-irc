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
    OSCaseMapping,
    CaseMapping,
    IrcAsciiExt,
    OwnedIrcAsciiExt,
    AsciiCaseMapping,
    Rfc1459CaseMapping,
    StrictRfc1459CaseMapping,
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

pub use self::parse::IrcMsg;

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
