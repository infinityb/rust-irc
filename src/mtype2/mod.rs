//! The submodules are named by who emits the message. e.g. clients
//! emit messages constructed in `client` and servers will emit messages
//! constructed in `server`.

use std::io;

use ::IrcMsg;

#[macro_use]
mod macros;

pub mod server;
pub mod client;

pub trait FromIrcMsg: Sized {
    type Err;

    /// This never allocates, but may check the underlying storage
    /// for well-formedness.
    fn from_irc_msg(msg: &IrcMsg) -> Result<Self, Self::Err>;
}

fn cursor_chk_error(err: io::Error) -> Result<(), ()> {
    match err {
        ref err if err.kind() == io::ErrorKind::WriteZero => Err(()),
        _ => panic!(),
    }
}
