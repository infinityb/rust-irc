use parse::IrcMsgNew as IrcMsg;


pub trait FromIrcMsg {
	fn from_irc_msg(msg: IrcMsg) -> Result<Self, IrcMsg>;
}
