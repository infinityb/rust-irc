use std::str;

use parse::IrcMsgNew as IrcMsg;
use irccase::IrcAsciiExt;
use message_types::util::is_valid_target;
use message_types::traits::FromIrcMsg;

pub struct Kick(IrcMsg);

impl Kick {
	pub fn new_opt(channel: &str, target: &str, message: &[u8]) -> Option<Kick> {
		if !is_valid_target(channel) {
			return None
		}
		if !is_valid_target(target) {
			return None
		}

		let mut buf = Vec::with_capacity(6 + target.len() + message.len());
		buf.push_all(b"KICK ");
		buf.push_all(channel.as_bytes());
		buf.push_all(b" ");
		buf.push_all(target.as_bytes());
		buf.push_all(b" :");
		buf.push_all(message);

		IrcMsg::new(buf).ok().and_then(|msg| Some(Kick(msg)))
	}

	pub fn new(channel: &str, target: &str, message: &[u8]) -> Kick {
		Kick::new_opt(channel, target, message).expect("invalid KICK target")
	}

	pub fn get_channel(&self) -> &str {
		let Kick(ref msg) = *self;
		unsafe { str::from_utf8_unchecked(&msg[0]) }
	}

	pub fn get_target(&self) -> &str {
		let Kick(ref msg) = *self;
		unsafe { str::from_utf8_unchecked(&msg[1]) }
	}

	pub fn unwrap(self) -> IrcMsg {
		let Kick(msg) = self;
		msg
	}
}

impl FromIrcMsg for Kick {
	fn from_irc_msg(msg: IrcMsg) -> Option<Kick> {
		if !msg.get_command().eq_ignore_irc_case("KICK") {
			return None;
		}
		if msg.len() <= 2 {
			return None;
		}
		Some(Kick(msg))
	}
}



pub struct Notice(IrcMsg);

impl Notice {
	/// send a NOTICE to `target` with the message `message`
	pub fn new_opt(&self, target: &str, message: &[u8]) -> Option<Notice> {
		if !is_valid_target(target) {
			return None
		}

		let mut buf = Vec::with_capacity(9 + target.len() + message.len());
		buf.push_all(b"NOTICE ");
		buf.push_all(target.as_bytes());
		buf.push_all(b" :");
		buf.push_all(message);

		// We've generated a valid message, no failure possible.
		Some(Notice(IrcMsg::new(buf).ok().expect("Programmer error")))
	}

	pub fn unwrap(self) -> IrcMsg {
		let Notice(msg) = self;
		msg
	}
}

impl FromIrcMsg for Notice {
	fn from_irc_msg(_msg: IrcMsg) -> Option<Notice> {
		unimplemented!();
	}
}


pub struct Privmsg(IrcMsg);

impl Privmsg {
	/// make a PRIVMSG with one argument
	pub fn new_opt(&self, target: &str, message: &[u8]) -> Option<Privmsg> {
		if !is_valid_target(target) {
			return None
		}

		let mut buf = Vec::with_capacity(10 + target.len() + message.len());
		buf.push_all(b"PRIVMSG ");
		buf.push_all(target.as_bytes());
		buf.push_all(b" :");
		buf.push_all(message.as_slice());

		// We've generated a valid message, no failure possible.
		Some(Privmsg(IrcMsg::new(buf).ok().expect("Programmer error")))
	}

	pub fn unwrap(self) -> IrcMsg {
		let Privmsg(msg) = self;
		msg
	}
}

impl FromIrcMsg for Privmsg {
	fn from_irc_msg(_msg: IrcMsg) -> Option<Privmsg> {
		unimplemented!();
	}
}
