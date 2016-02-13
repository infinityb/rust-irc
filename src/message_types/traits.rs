use ::parse::old_parse::IrcMsg;


pub trait FromIrcMsg: Sized {
    fn from_irc_msg(msg: IrcMsg) -> Result<Self, IrcMsg>;
}
