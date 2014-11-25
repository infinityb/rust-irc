use std::str;

use parse::IrcMsgNew as IrcMsg;
use irccase::IrcAsciiExt;
use message_types::util::is_valid_target;
use message_types::traits::FromIrcMsg;
pub use message_types::common::{
	Kick,
	Privmsg,
};

pub enum OutgoingMsg {
	Join(Join),
}

pub struct Join(IrcMsg);

impl Join {
	/// JOIN a channel `target` with the parting message `message`.
	/// returns `None` if target is not a valid target.
	pub fn new_opt(target: &str, key: &[u8]) -> Option<Join> {
		if !is_valid_target(target) {
			return None
		}

		let mut buf = Vec::with_capacity(6 + target.len() + key.len());
		buf.push_all(b"JOIN ");
		buf.push_all(target.as_bytes());
		buf.push_all(b" ");
		buf.push_all(key);

		IrcMsg::new(buf).ok().and_then(|msg| Some(Join(msg)))
	}

	/// JOIN a channel `target` with the parting message `message`, panicking
	/// if the target is not valid.
	pub fn new(target: &str, key: &[u8]) -> Join {
		Join::new_opt(target, key).expect("invalid JOIN target")
	}

	/// The target channel of this JOIN
	pub fn get_target(&self) -> &str {
		let Join(ref msg) = *self;
		// UTF8 ensured by from_irc_msg and new_opt
		unsafe { str::from_utf8_unchecked(&msg[0]) }
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
		if msg.len() <= 2 {
			return None;
		}
		if !str::is_utf8(&msg[0]) {
			return None;
		}
		Some(Join(msg))
	}
}


pub struct Part(IrcMsg);

impl Part {
	/// PART a channel `target` with the parting message `message`
	pub fn new_opt(&self, target: &str, message: &[u8]) -> Option<Part> {
		if !is_valid_target(target) {
			return None
		}

		let mut buf = Vec::with_capacity(7 + target.len() + message.len());
		buf.push_all(b"PART ");
		buf.push_all(target.as_bytes());
		buf.push_all(b" :");
		buf.push_all(message);

		// We've generated a valid message, no failure possible.
		Some(Part(IrcMsg::new(buf).ok().expect("Programmer error")))
	}

	/// PART a channel `target` with the parting message `message`, panicking
	/// if the target is not valid.
	pub fn new(target: &str, message: &[u8]) -> Join {
		Join::new_opt(target, message).expect("invalid JOIN target")
	}
}

impl FromIrcMsg for Part {
	fn from_irc_msg(_msg: IrcMsg) -> Option<Part> {
		unimplemented!();
	}
}


pub struct Ping(IrcMsg);

impl Ping {
	/// make a PING with one argument
	pub fn new_one(&self, first: &str) -> Option<Ping> {
		if !is_valid_target(first) {
			return None
		}

		let mut buf = Vec::with_capacity(5 + first.len());
		buf.push_all(b"PING ");
		buf.push_all(first.as_bytes());

		// We've generated a valid message, no failure possible.
		Some(Ping(IrcMsg::new(buf).ok().expect("Programmer error")))
	}

	/// make a PING with two arguments
	pub fn new_two(&self, first: &str, second: &str) -> Option<Ping> {
		if !is_valid_target(first) {
			return None
		}
		if !is_valid_target(second) {
			return None
		}
	
		let mut buf = Vec::with_capacity(6 + first.len() + second.len());

		buf.push_all(b"PING ");
		buf.push_all(first.as_bytes());
		buf.push_all(b" ");
		buf.push_all(second.as_bytes());

		// We've generated a valid message, no failure possible.
		Some(Ping(IrcMsg::new(buf).ok().expect("Programmer error")))
	}
}

impl FromIrcMsg for Ping {
	fn from_irc_msg(_msg: IrcMsg) -> Option<Ping> {
		unimplemented!();
	}
}

