use std::str::{is_utf8, raw, MaybeOwned};
use std::fmt;
use util::{StringSlicer, OptionalStringSlicer};

use irccase::IrcAsciiExt;

static CHANNEL_PREFIX_CHARS: [char, ..4] = ['&', '#', '+', '!'];

// Commands which target a msgtarget or channel
static CHANNEL_TARGETED_COMMANDS: [&'static str, ..6] = [
	"KICK",
	"PART",
	"MODE",
	"PRIVMSG",
	"NOTICE",
	"TOPIC"
];


/// Whether or not a command name is allowed to target a channel
pub fn can_target_channel(identifier: &str) -> bool {
	for &command in CHANNEL_TARGETED_COMMANDS.as_slice().iter() {
		if command.eq_ignore_irc_case(identifier) {
			return true;
		}
	}
	false
}

/// Determines whether or not an identifier is a channel, by checking
/// the first character.
pub fn is_channel(identifier: &str) -> bool {
	if identifier.char_len() == 0 {
		return false;
	}
	for character in CHANNEL_PREFIX_CHARS.iter() {
		if identifier.char_at(0) == *character {
			return true;
		}
	}
	false
}


enum IrcParserState {
	Initial,
	Prefix,
	CommandStart,
	Command,
	ArgStart,
	Arg,
	RestArg,
	ArgOverflow,
	Failed,
	EndOfLine,
}

struct IrcParser {
	byte_idx: u32,
	prefix_start: u32,
	prefix_end: u32,
	command_start: u32,
	command_end: u32,
	arg_len: u32,
	arg_start: u32,
	args: [(u32, u32), ..15],
	state: IrcParserState
}

#[deriving(Show, PartialEq, Eq)]
pub enum ParseError {
	InvalidMessage(&'static str),
	EncodingError,
}

impl IrcParser {
	fn new() -> IrcParser {
		IrcParser {
			byte_idx: 0,
			prefix_start: 0,
			prefix_end: 0,
			command_start: 0,
			command_end: 0,
			arg_len: 0,
			arg_start: 0,
			args: [(0, 0), ..15],
			state: IrcParserState::Initial,
		}
	}

	#[inline]
	fn push_byte(&mut self, byte: u8) {
		self.state = match (self.state, byte) {
			(IrcParserState::Initial, b' ') => IrcParserState::Initial,
			(IrcParserState::Initial, b':') => {
				self.prefix_start = self.byte_idx + 1;
				IrcParserState::Prefix
			}
			(IrcParserState::Initial, _) => {
				self.command_start = self.byte_idx;
				IrcParserState::Command
			},

			(IrcParserState::Prefix, b' ') => {
				self.prefix_end = self.byte_idx;
				IrcParserState::CommandStart
			},
			(IrcParserState::Prefix, _) => IrcParserState::Prefix,

			(IrcParserState::CommandStart, b' ') => IrcParserState::CommandStart,
			(IrcParserState::CommandStart, _) => {
				self.command_start = self.byte_idx;
				IrcParserState::Command
			}

			(IrcParserState::Command, b' ') => {
				self.command_end = self.byte_idx;
				IrcParserState::ArgStart
			}
			(IrcParserState::Command, _) => IrcParserState::Command,

			(IrcParserState::ArgStart, b' ') => IrcParserState::ArgStart,
			(IrcParserState::ArgStart, b':') => {
				self.arg_start = self.byte_idx + 1;
				IrcParserState::RestArg
			}
			(IrcParserState::ArgStart, _) => {
				self.arg_start = self.byte_idx;
				IrcParserState::Arg
			}

			(IrcParserState::Arg, b' ') => {
				self.args[self.arg_len as uint] = (self.arg_start, self.byte_idx);
				self.arg_len += 1;
				self.arg_start = 0;
				if self.arg_len == 15 {
					IrcParserState::ArgOverflow
				} else {
					IrcParserState::ArgStart
				}
			}
			(IrcParserState::Arg, _) => IrcParserState::Arg,

			(IrcParserState::RestArg, b'\n') => {
				self.args[self.arg_len as uint] = (self.arg_start, self.byte_idx - 1);
				self.arg_len += 1;
				self.arg_start = 0;
				if self.arg_len == 15 {
					IrcParserState::ArgOverflow
				} else {
					IrcParserState::EndOfLine
				}
			}
			(IrcParserState::RestArg, _) => IrcParserState::RestArg,

			(IrcParserState::ArgOverflow, _) => IrcParserState::ArgOverflow,
			(IrcParserState::Failed, _) => IrcParserState::Failed,
			(IrcParserState::EndOfLine, _) => IrcParserState::EndOfLine,
		};
		self.byte_idx += 1;
	}

	#[inline]
	fn finish(&mut self) -> Result<(), ParseError> {
		match self.state {
			IrcParserState::Initial => Err(ParseError::InvalidMessage("too short")),
			IrcParserState::Prefix => Err(ParseError::InvalidMessage("too short")),
			IrcParserState::CommandStart => Err(ParseError::InvalidMessage("too short")),
			IrcParserState::Command => Err(ParseError::InvalidMessage("too short")),
			IrcParserState::Failed => Err(ParseError::InvalidMessage("failed")),
			IrcParserState::ArgOverflow => Err(ParseError::InvalidMessage("too many arguments")),
			IrcParserState::ArgStart => Ok(()),
			IrcParserState::EndOfLine => Ok(()),
			IrcParserState::Arg | IrcParserState::RestArg => {
				self.args[self.arg_len as uint] = (self.arg_start, self.byte_idx);
				self.arg_len += 1;
				Ok(())
			}
		}
	}

	fn parse(message: Vec<u8>) -> Result<IrcMsg, ParseError> {
		let mut parser = IrcParser::new();
		for value in message.as_slice().iter() {
			parser.push_byte(*value);
		}
		match parser.finish() {
			Ok(()) => (),
			Err(err) => return Err(err)
		};
		assert_eq!(parser.byte_idx as uint, message.len());
		let mut parsed = IrcMsg {
			data: message,
			prefix: (parser.prefix_start, parser.prefix_end),
			command: (parser.command_start, parser.command_end),
			arg_len: 0,
			args: [(0, 0), ..15]
		};

		parsed.arg_len = parser.arg_len;
		for i in range(0, parsed.arg_len) {
			parsed.args[i as uint] = parser.args[i as uint];
		}

		// Carriage return removal
		let last_idx = (parsed.arg_len - 1) as uint;
		let (arg_start, arg_end) = parsed.args[last_idx];
		if parsed.data[arg_end as uint - 1] == b'\r' {
			parsed.args[last_idx] = (arg_start, arg_end - 1) ;
		}

		Ok(parsed)
	}
}

// RFC 2812 2.3 Messages
// 
// Each IRC message may consist of up to three main parts:
// the prefix (optional), the command, and the command
// parameters (of which there may be up to 15). The prefix,
// command, and all parameters are separated by one (or 
// more) ASCII space character(s) (0x20).

#[experimental]
pub struct IrcMsg {
	data: Vec<u8>,
	prefix: (u32, u32),
	command: (u32, u32),
	arg_len: u32,
	args: [(u32, u32), ..15],
}

impl IrcMsg {
	pub fn new(data: Vec<u8>) -> Result<IrcMsg, ParseError> {
		let parsed = match IrcParser::parse(data) {
			Ok(parsed) => parsed,
			Err(err) => return Err(err)
		};
		if is_utf8(parsed.get_prefix_raw()) {
			return Err(ParseError::EncodingError)
		}
		if is_utf8(parsed.get_command()) {
			return Err(ParseError::EncodingError)
		}
		Ok(parsed)
	}

	pub fn has_prefix(&self) -> bool {
		let (prefix_start, prefix_end) = self.prefix;
		prefix_end > prefix_start
	}

	pub fn get_prefix_raw(&self) -> &[u8] {
		let (prefix_start, prefix_end) = self.prefix;
		self.data[prefix_start as uint..prefix_end as uint]
	}

	pub fn get_prefix<'a>(&'a self) -> IrcMsgPrefix<'a> {
		let prefix_ref = unsafe { raw::from_utf8(self.get_prefix_raw()) };
		IrcMsgPrefix::new(prefix_ref.into_maybe_owned())
	}

	pub fn get_command(&self) -> &[u8] {
		let (command_start, command_end) = self.command;
		self.data[command_start as uint..command_end as uint]
	}

	pub fn get_args(&self) -> Vec<&[u8]> {
		let mut out = Vec::with_capacity(self.arg_len as uint);
		for i in range(0, self.arg_len as uint) {
			let (arg_start, arg_end) = self.args[i];
			out.push(self.data[arg_start as uint..arg_end as uint]);
		}
		out
	}

	pub fn unwrap(self) -> Vec<u8> {
		self.data
	}
}


#[cfg(test)]
mod tests {
	use super::{IrcParser, ParseError};
	use test::Bencher;

	#[test]
	fn test_basics() {
		{
			let example: Vec<_> = b":prefix PING  foo bar baz".iter().map(|&x| x).collect();

			let parsed = match IrcParser::parse(example) {
				Ok(parsed) => parsed,
				Err(err) => panic!("err: {}", err)
			};
			assert!(parsed.has_prefix());
			assert_eq!(parsed.get_prefix_raw(), b"prefix");
			assert_eq!(parsed.get_command(), b"PING");
			assert_eq!(parsed.get_args().as_slice(), [b"foo", b"bar", b"baz"].as_slice());
		}

		{
			let example: Vec<_> = b"PING a b c d e f g h i j k l m n o p q r s t u v w x y z\n".iter().map(|&x| x).collect();
			assert_eq!(IrcParser::parse(example).err(), Some(ParseError::InvalidMessage("too many arguments")));
		}

		{
			let example: Vec<_> = b":prefix PING  foo :bar baz\r\n".iter().map(|&x| x).collect();

			let parsed = match IrcParser::parse(example) {
				Ok(parsed) => parsed,
				Err(err) => panic!("err: {}", err)
			};
			assert!(parsed.has_prefix());
			assert_eq!(parsed.get_prefix_raw(), b"prefix");
			assert_eq!(parsed.get_command(), b"PING");
			assert_eq!(parsed.get_args().as_slice(), [b"foo", b"bar baz"].as_slice());
		}
	}

	#[bench]
	fn irc_parser_msg(b: &mut Bencher) {
		let message = b":irc.rizon.no 372 cooldude` :- o No takeovers\n";
	    b.iter(|| {
	    	let mut parser = IrcParser::new();
			for value in message.as_slice().iter() {
				parser.push_byte(*value);
			}
			assert!(parser.finish().is_ok());
	    })
	}
}


#[deriving(Clone)]
pub struct PrefixSlicer {
	pub nick_idx_pair: OptionalStringSlicer,
	username_idx_pair: OptionalStringSlicer,
	hostname_idx_pair: StringSlicer
}

impl PrefixSlicer {
	pub fn new(prefix: &str) -> PrefixSlicer {
		let idx_pair = match prefix.find('!') {
            Some(exc_idx) => match prefix[exc_idx+1..].find('@') {
                Some(at_idx) => Some((exc_idx, exc_idx + at_idx + 1)),
                None => None
            },
            None => None
        };

        match idx_pair {
            Some((exc_idx, at_idx)) => PrefixSlicer {
                nick_idx_pair: OptionalStringSlicer::new_some(0, exc_idx),
                username_idx_pair: OptionalStringSlicer::new_some(exc_idx + 1, at_idx),
                hostname_idx_pair: StringSlicer::new(at_idx + 1, prefix.len())
            },
            None => PrefixSlicer {
                nick_idx_pair: OptionalStringSlicer::new_none(),
                username_idx_pair: OptionalStringSlicer::new_none(),
                hostname_idx_pair: StringSlicer::new(0, prefix.len())
            }
        }
	}

	pub fn apply<'a>(&self, prefix: MaybeOwned<'a>) -> IrcMsgPrefix<'a> {
		IrcMsgPrefix {
			data: prefix,
			slicer: self.clone()
		}
	}
}

/// An IRC prefix, which identifies the source of a message.
#[deriving(Clone)]
pub struct IrcMsgPrefix<'a> {
	data: MaybeOwned<'a>,
	slicer: PrefixSlicer
}

impl<'a> IrcMsgPrefix<'a> {
	/// Parse a MaybeOwned into a IrcMsgPrefix
	pub fn new(s: MaybeOwned<'a>) -> IrcMsgPrefix {
		let slicer = PrefixSlicer::new(s.as_slice());
		IrcMsgPrefix {
			data: s,
			slicer: slicer
		}
	}

	/// The nick component of a prefix
	pub fn nick(&'a self) -> Option<&'a str> {
		self.slicer.nick_idx_pair.slice_on(self.data.as_slice())
	}

	/// The username component of a prefix
	pub fn username(&'a self) -> Option<&'a str> {
		self.slicer.username_idx_pair.slice_on(self.data.as_slice())
	}

	/// The hostname component of a prefix
	pub fn hostname(&'a self) -> &'a str {
		self.slicer.hostname_idx_pair.slice_on(self.data.as_slice())
	}

	/// Get the protocol representation as a slice
	pub fn as_slice(&self) -> &str {
		self.data.as_slice()
	}

	/// Get an owned copy
	pub fn into_owned(&self) -> IrcMsgPrefix<'static> {
		IrcMsgPrefix {
			data: self.data.to_string().into_maybe_owned(),
			slicer: self.slicer.clone()
		}
	}

	pub fn with_nick(&self, nick: &str) -> Option<IrcMsgPrefix<'static>> {
		match (self.nick(), self.username(), self.hostname()) {
			(Some(_), Some(username), hostname) => {
				let prefix_data = format!("{}!{}@{}", nick, username, hostname);
				Some(IrcMsgPrefix::new(prefix_data.into_maybe_owned()))
			},
			_ => None
		}
	}
}


impl<'a> PartialEq for IrcMsgPrefix<'a> {
    fn eq(&self, other: &IrcMsgPrefix<'a>) -> bool {
    	self.as_slice() == other.as_slice()
    }
}
impl<'a> Eq for IrcMsgPrefix<'a> {}

impl<'a> fmt::Show for IrcMsgPrefix<'a> {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "IrcMsgPrefix::new({})", self.as_slice())
    }
}