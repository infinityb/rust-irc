use super::IrcMsg;

/// Messages that come from the server
pub mod server;


/// Messages that come from the client
pub mod client;


pub trait FromIrcMsg: Sized {
    fn from_irc_msg(msg: IrcMsg) -> Result<Self, IrcMsg>;
}
