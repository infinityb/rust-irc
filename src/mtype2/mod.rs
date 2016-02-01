use ::parse::parse2::IrcMsg;

pub mod server;

pub mod client;

pub trait FromIrcMsg: Sized {
    type Err;

    /// This never allocates, but may check the underlying storage
    /// for well-formedness.
    fn from_irc_msg(msg: &IrcMsg) -> Result<Self, Self::Err>;
}
