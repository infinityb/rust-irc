#![crate_name = "irc"]
#![crate_type = "dylib"]
#![license = "MIT/ASL2"]
#![feature(if_let, slicing_syntax, globs, phase)]

#![allow(dead_code)]
#![deny(unused_must_use, warnings, unused_variables, unused_mut)]


#[phase(plugin, link)] extern crate log;
extern crate time;
extern crate serialize;

pub use self::message::IrcMessage;

pub use self::connection::IrcConnection;
pub use self::connection::RawWrite;

pub use self::event::{
	IrcEvent,
	IrcEventMessage,
	IrcEventJoinBundle,
	IrcEventWhoBundle,
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

pub use self::parse::{
    IrcMsgPrefix,
};


mod numerics;
mod connection;
mod message;
mod watchers;
mod core_plugins;

#[experimental = "Subject to being moved"]
/// Experimental utility code
pub mod util;

#[experimental = "Subject to being moved"]
/// Experimental parsing code
pub mod parse;

/// Event types
mod event;
