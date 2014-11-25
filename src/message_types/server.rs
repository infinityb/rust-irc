use std::str;

use parse::is_full_prefix;
use parse::IrcMsgNew as IrcMsg;
use irccase::IrcAsciiExt;
use message_types::traits::FromIrcMsg;
pub use message_types::common::{
	Kick,
	Privmsg,
};


pub enum IncomingMsg {
	Join(Join),
}

pub struct Join(IrcMsg);

impl Join {
	pub fn get_channel(&self) -> &str {
		let Join(ref msg) = *self;
		unsafe { str::from_utf8_unchecked(&msg[0]) }
	}

	pub fn get_nick<'a>(&'a self) -> &'a str {
		let Join(ref msg) = *self;
		let prefix = msg.get_prefix_str();
		prefix[..prefix.find('!').unwrap()]
	}

	pub fn unwrap(self) -> IrcMsg {
		let Join(msg) = self;
		msg
	}
}

impl FromIrcMsg for Join {
	fn from_irc_msg(msg: IrcMsg) -> Option<Join> {
		if !msg.get_command().eq_ignore_irc_case("JOIN") {
			return None;
		}
		if msg.len() == 0 {
			warn!("Invalid message: Not enough arguments {}", msg.len());
			return None;
		}
		if !is_full_prefix(msg.get_prefix_str()) {
			warn!("Invalid message: Insufficient prefix `{}`", msg.get_prefix_str());
			return None;
		}
		Some(Join(msg))
	}
}


pub struct Numeric(IrcMsg, u16);

impl Numeric {
	pub fn unwrap(self) -> IrcMsg {
		let Numeric(msg, _) = self;
		msg
	}
}

impl FromIrcMsg for Numeric {
	fn from_irc_msg(msg: IrcMsg) -> Option<Numeric> {
		let numeric: u16 = match from_str(msg.get_command()) {
			Some(numeric) => numeric,
			None => return None
		};
		Some(Numeric(msg, numeric))
	}
}


pub struct Ping(IrcMsg);

impl Ping {
	pub fn unwrap(self) -> IrcMsg {
		let Ping(msg) = self;
		msg
	}
}

impl FromIrcMsg for Ping {
	fn from_irc_msg(_msg: IrcMsg) -> Option<Ping> {
		unimplemented!();
	}
}
