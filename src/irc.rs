#![crate_name = "irc"]
#![crate_type = "dylib"]
#![license = "MIT"]
#![feature(if_let, slicing_syntax, globs)]

extern crate time;
extern crate serialize;

pub use self::message::{
    IrcMessage,
    IrcProtocolMessage
};

pub use self::connection::IrcConnection;

pub use self::watchers::{
    MessageWatcher,
    JoinBundler,
    RegisterError,
    RegisterErrorType,
    JoinResult,
    JoinError,
};

pub mod numerics;
pub mod connection;
pub mod message;
pub mod watchers;
pub mod core_plugins;
pub mod util;
pub mod parse;