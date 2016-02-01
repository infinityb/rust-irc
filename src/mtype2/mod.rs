//! The submodules are named by who emits the message. e.g. clients
//! emit messages constructed in `client` and servers will emit messages
//! constructed in `server`.

use ::parse::parse2::IrcMsg;

pub mod server;

pub mod client;

pub trait FromIrcMsg: Sized {
    type Err;

    /// This never allocates, but may check the underlying storage
    /// for well-formedness.
    fn from_irc_msg(msg: &IrcMsg) -> Result<Self, Self::Err>;
}
