
use std::str::MaybeOwned;
use util::{StringSlicer, OptionalStringSlicer};

struct IrcMsg<'a> {
	// RFC1459: max 512 bytes
	data: MaybeOwned<'a>,
	prefix: OptionalStringSlicer,
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
	pub fn from_str(msg_text: &'a str) -> Option<IrcMsg<'a>> {
		IrcMsg::new(msg_text.into_maybe_owned())
	}

	pub fn new(msg_text: MaybeOwned<'a>) -> Option<IrcMsg<'a>> {
		let mut cur_idx = 0;

		let (tmp, prefix_ss) = extract_prefix(cur_idx, msg_text.as_slice());
		cur_idx = consume_spaces(tmp, msg_text.as_slice());
		
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
			command: command,
			args: args
		})
	}

	#[inline]
	pub fn get_prefix<'a>(&'a self) -> Option<&'a str> {
		self.prefix.slice_on(self.data.as_slice())
	}

	#[inline]
	pub fn get_command<'a>(&'a self) -> &'a str {
		self.command.slice_on(self.data.as_slice())
	}

	#[inline]
	pub fn get_args<'a>(&'a self) -> Vec<&'a str> {
		self.args.iter().map(|ss: &StringSlicer| {
			ss.slice_on(self.data.as_slice())
		}).collect()
	}
}

fn consume_spaces(start: uint, text: &str) -> uint {
	let mut idx = 0;

	let mut tmp = text[start..];
	loop {
		match tmp.slice_shift_char() {
			(Some(' '), rest) => {
				idx += 1;
				tmp = rest;
			},
			(Some(_), rest) => break,
			(None, rest) => return start + idx
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
		assert_eq!(msg.get_prefix(), None);
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
		assert_eq!(msg.get_prefix(), None);
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
		assert_eq!(msg.get_prefix(), Some("nick!user@host"));
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
		assert_eq!(msg.get_prefix(), Some("nick!user@host"));
		assert_eq!(msg.get_command(), "PING");
		assert_eq!(msg.get_args().len(), 2);
		assert_eq!(msg.get_args()[0], "server1");
		assert_eq!(msg.get_args()[1], "server2");
	}


}