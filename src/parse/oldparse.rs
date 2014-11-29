use std::str::{MaybeOwned, FromStr};
use util::{StringSlicer, OptionalStringSlicer};
use parse::parse::{IrcMsgPrefix, PrefixSlicer};
use std::str::IntoMaybeOwned;

/// Represents any syntactically valid IRC message.
/// No semantic checking is applied.
#[deriving(Clone)]
pub struct IrcMsg<'a> {
	// RFC1459: max 512 bytes
	data: MaybeOwned<'a>,
	prefix: OptionalStringSlicer,
	prefix_extra: Option<PrefixSlicer>,
	command: StringSlicer,
	args: Vec<StringSlicer>
}

// RFC1459:
// Each IRC message may consist of up to three main parts:
// the prefix (optional), the command, and the command
// parameters (of which there may be up to 15). The prefix,
// command, and all parameters are separated by one (or 
// more) ASCII space character(s) (0x20).

impl<'a> IrcMsg<'a> {
	#[inline]
	fn from_str(msg_text: &'a str) -> Option<IrcMsg<'a>> {
		// TODO: find out how to make this allocation-free again
		IrcMsg::new(msg_text.into_maybe_owned())
	}

	/// Parses a string into an optional IrcMsg.  If the string is not syntactically valid,
	/// None is returned.
	pub fn new(msg_text: MaybeOwned<'a>) -> Option<IrcMsg<'a>> {
		let mut cur_idx = 0;

		let (tmp, prefix_ss) = extract_prefix(cur_idx, msg_text.as_slice());
		cur_idx = consume_spaces(tmp, msg_text.as_slice());
		
		let prefix_extra = match prefix_ss.slice_on(msg_text.as_slice()) {
			Some(pref) => Some(PrefixSlicer::new(pref)),
			None => None
		};

		let (tmp, command) = extract_word(cur_idx, msg_text.as_slice());
		cur_idx = consume_spaces(tmp, msg_text.as_slice());

		let mut args: Vec<StringSlicer> = Vec::new();

		while msg_text.as_slice()[cur_idx..] != "" {
			let (tmp, arg) = extract_arg(cur_idx, msg_text.as_slice());
			cur_idx = consume_spaces(tmp, msg_text.as_slice());
			args.push(arg);
		}

		Some(IrcMsg {
			data: msg_text,
			prefix: prefix_ss,
			prefix_extra: prefix_extra,
			command: command,
			args: args
		})
	}

	#[inline]
	/// The prefix which identifies the source of a message, as a string slice
	pub fn get_prefix_raw(&'a self) -> Option<&'a str> {
		self.prefix.slice_on(self.data.as_slice())
	}

	#[inline]
	/// The prefix which identifies the source of a message
	pub fn get_prefix(&'a self) -> Option<IrcMsgPrefix<'a>> {
		let prefix = match self.prefix.slice_on(self.data.as_slice()) {
			Some(prefix) => prefix,
			None => return None
		};
		match self.prefix_extra {
			Some(extra) => Some(extra.apply(prefix.into_maybe_owned())),
			None => None
		}
	}

	#[inline]
	/// The command name of a message as received i.e. without normalisation.
	pub fn get_command(&'a self) -> &'a str {
		self.command.slice_on(self.data.as_slice())
	}

	#[inline]
	/// The arguments of a message
	pub fn get_args(&'a self) -> Vec<&'a str> {
		self.args.iter().map(|ss: &StringSlicer| {
			ss.slice_on(self.data.as_slice())
		}).collect()
	}

	#[inline]
	/// The nick of the user a message came from, if any.
	pub fn source_nick(&'a self) -> Option<&'a str> {
		let slicer = match self.prefix_extra {
			Some(pe) => pe.nick_idx_pair.slice_from_opt(&self.prefix),
			None => return None
		};
		slicer.slice_on(self.data.as_slice())
	}
}


impl<'a> FromStr for IrcMsg<'a> {
	#[inline]
	fn from_str(msg_text: &str) -> Option<IrcMsg<'a>> {
		// TODO: find out how to make this allocation-free again
		IrcMsg::new(msg_text.to_string().into_maybe_owned())
	}
}

fn consume_spaces(start: uint, text: &str) -> uint {
	let mut idx = 0;

	let mut tmp = text[start..];
	loop {
		match tmp.slice_shift_char() {
			Some((' ', rest)) => {
				idx += 1;
				tmp = rest;
			},
			Some((_, _)) => break,
			None => return start + idx
		}
	}
	start + idx
}

fn extract_prefix(start: uint, text: &str) -> (uint, OptionalStringSlicer) {
	let tmp = text[start..];
	if tmp.starts_with(":") {
		let end_idx = match tmp.find(' ') {
			Some(idx) => idx,
			None => tmp.len()
		};
		(start + end_idx, OptionalStringSlicer::new_some(start + 1, start + end_idx))
	} else {
		(start, OptionalStringSlicer::new_none())
	}
}

fn extract_word(start: uint, text: &str) -> (uint, StringSlicer) {
	let tmp = text[start..];
	let end_idx = match tmp.find(' ') {
		Some(idx) => idx,
		None => tmp.len()
	};
	(start + end_idx, StringSlicer::new(start, start + end_idx))
}

fn extract_arg(start: uint, text: &str) -> (uint, StringSlicer) {
	let tmp = text[start..];
	let (start_offset, end_idx) = if tmp.starts_with(":") {
		(1, tmp.len())
	} else {
		match tmp.find(' ') {
			Some(idx) => (0, idx),
			None => (0, tmp.len())
		}
	};
	(start + end_idx, StringSlicer::new(start + start_offset, start + end_idx))
}

#[test]
fn test_irc_msg() {
	let pings_noprefix_1server = vec![
		"PING server1",
		"PING  server1",
		"PING server1 ",
		"PING      server1"
	];
	for msg_text in pings_noprefix_1server.into_iter() {
		println!("--!!-- running with ``{}`` --!!--", msg_text);
		let msg = IrcMsg::from_str(msg_text);
		assert!(msg.is_some());
		let msg = msg.unwrap();
		assert_eq!(msg.get_prefix_raw(), None);
		assert!(msg.get_prefix().is_none());
		assert_eq!(msg.get_command(), "PING");
		assert_eq!(msg.get_args().len(), 1);
		assert_eq!(msg.get_args()[0], "server1");
	}

	let pings_noprefix_2server = vec![
		"PING server1 :server2",
		"PING server1 :server2",
		"PING  server1 :server2",
		"PING server1  :server2",
		"PING    server1  :server2"
	];
	for msg_text in pings_noprefix_2server.into_iter() {
		let msg = IrcMsg::from_str(msg_text);
		assert!(msg.is_some());
		let msg = msg.unwrap();
		assert_eq!(msg.get_prefix_raw(), None);
		assert!(msg.get_prefix().is_none());
		assert_eq!(msg.get_command(), "PING");
		assert_eq!(msg.get_args().len(), 2);
		assert_eq!(msg.get_args()[0], "server1");
		assert_eq!(msg.get_args()[1], "server2");
	}

	let pings_prefix_1server = vec![
		":nick!user@host PING server1",
		":nick!user@host  PING server1",
		":nick!user@host PING  server1 ",  // is this valid?
		":nick!user@host  PING server1",
		":nick!user@host  PING    server1"
	];
	for msg_text in pings_prefix_1server.into_iter() {
		let msg = IrcMsg::from_str(msg_text);
		assert!(msg.is_some());
		let msg = msg.unwrap();
		assert_eq!(msg.get_prefix_raw(), Some("nick!user@host"));
		assert!(msg.get_prefix().is_some());
		assert_eq!(msg.get_command(), "PING");
		assert_eq!(msg.get_args().len(), 1);
		assert_eq!(msg.get_args()[0], "server1");
	}

	let pings_prefix_2server = vec![
		":nick!user@host PING server1 :server2",
		":nick!user@host  PING server1 :server2",
		":nick!user@host PING  server1 :server2",
		":nick!user@host  PING server1  :server2",
		":nick!user@host  PING    server1  :server2"
	];
	for msg_text in pings_prefix_2server.into_iter() {
		let msg = IrcMsg::from_str(msg_text);
		assert!(msg.is_some());
		let msg = msg.unwrap();
		assert_eq!(msg.get_prefix_raw(), Some("nick!user@host"));
		assert!(msg.get_prefix().is_some());
		assert_eq!(msg.get_command(), "PING");
		assert_eq!(msg.get_args().len(), 2);
		assert_eq!(msg.get_args()[0], "server1");
		assert_eq!(msg.get_args()[1], "server2");
	}
}
