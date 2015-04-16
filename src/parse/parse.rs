use std::fmt;
use std::ops::Index;
use std::borrow::{Cow, IntoCow};

use util::{StringSlicer, OptionalStringSlicer};

use irccase::IrcAsciiExt;

static CHANNEL_PREFIX_CHARS: [char; 4] = ['&', '#', '+', '!'];

// Commands which target a msgtarget or channel
static CHANNEL_TARGETED_COMMANDS: [&'static str; 6] = [
    "KICK",
    "PART",
    "MODE",
    "PRIVMSG",
    "NOTICE",
    "TOPIC"
];


/// Whether or not a command name is allowed to target a channel
pub fn can_target_channel(identifier: &str) -> bool {
    for &command in CHANNEL_TARGETED_COMMANDS.iter() {
        if command.eq_ignore_irc_case(identifier) {
            return true;
        }
    }
    false
}

/// Determines whether or not an identifier is a channel, by checking
/// the first character.
pub fn is_channel(identifier: &str) -> bool {
    if identifier.chars().count() == 0 {
        return false;
    }
    for &character in CHANNEL_PREFIX_CHARS.iter() {
        if identifier.chars().next() == Some(character) {
            return true;
        }
    }
    false
}


enum PrefixCheckerState {
    Nick,
    User,
    Host,
}

/// Checks whether a prefix contains nick, username and host or not.
pub fn is_full_prefix(prefix: &str) -> bool {
    let mut state = PrefixCheckerState::Nick;
    for &byte in prefix.as_bytes().iter() {
        state = match (state, byte) {
            (PrefixCheckerState::Nick, b'!') => PrefixCheckerState::User,
            (PrefixCheckerState::Nick, _) => PrefixCheckerState::Nick,
            (PrefixCheckerState::User, b'@') => PrefixCheckerState::Host,
            (PrefixCheckerState::User, _) => PrefixCheckerState::User,
            (PrefixCheckerState::Host, _) => PrefixCheckerState::Host,
        };
    }
    match state {
        PrefixCheckerState::Host => true,
        _ => false
    }
}

#[derive(Copy, Clone)]
enum IrcParserState {
    Initial,
    Prefix,
    CommandStart,
    Command,
    ArgStart,
    Arg,
    RestArg,
    ArgOverflow,
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
    args: [(u32, u32); IRCMSG_MAX_ARGS],
    state: IrcParserState
}

#[derive(Debug, PartialEq, Eq)]
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
            args: [(0, 0); IRCMSG_MAX_ARGS],
            state: IrcParserState::Initial,
        }
    }

    fn finalize_arg(&mut self) -> Option<IrcParserState> {
        if self.arg_len as usize == IRCMSG_MAX_ARGS{
            return Some(IrcParserState::ArgOverflow);
        }
        self.args[self.arg_len as usize] = (self.arg_start, self.byte_idx);
        self.arg_len += 1;
        self.arg_start = 0;
        None
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
                match self.finalize_arg() {
                    Some(state) => state,
                    None => IrcParserState::ArgStart,
                }
            }
            (IrcParserState::Arg, b'\n') => {
                match self.finalize_arg() {
                    Some(state) => state,
                    None => IrcParserState::EndOfLine,
                }
            }
            (IrcParserState::Arg, _) => IrcParserState::Arg,

            (IrcParserState::RestArg, b'\n') => {
                match self.finalize_arg() {
                    Some(state) => state,
                    None => IrcParserState::EndOfLine,
                }
            }
            (IrcParserState::RestArg, _) => IrcParserState::RestArg,

            (IrcParserState::ArgOverflow, _) => IrcParserState::ArgOverflow,
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
            IrcParserState::ArgOverflow => Err(ParseError::InvalidMessage("too many arguments")),
            IrcParserState::ArgStart => Ok(()),
            IrcParserState::EndOfLine => Ok(()),
            IrcParserState::Arg | IrcParserState::RestArg => {
                self.args[self.arg_len as usize] = (self.arg_start, self.byte_idx);
                self.arg_len += 1;
                Ok(())
            }
        }
    }

    fn parse(message: Vec<u8>) -> Result<IrcMsg, ParseError> {
        let mut parser = IrcParser::new();
        for &value in message.iter() {
            parser.push_byte(value);
        }
        match parser.finish() {
            Ok(()) => (),
            Err(err) => return Err(err)
        };
        assert_eq!(parser.byte_idx as usize, message.len());

        let mut parsed = IrcMsg {
            data: message,
            prefix: (parser.prefix_start, parser.prefix_end),
            command: (parser.command_start, parser.command_end),
            arg_len: 0,
            args: [(0, 0); IRCMSG_MAX_ARGS]
        };

        parsed.arg_len = parser.arg_len;
        for i in 0..parsed.arg_len {
            parsed.args[i as usize] = parser.args[i as usize];
        }

        // Newline and Carriage return removal
        let last_idx = (parsed.arg_len - 1) as usize;
        let (arg_start, mut arg_end) = parsed.args[last_idx];

        if parsed.data[arg_end as usize - 1] == b'\r' {
            arg_end -= 1;
            parsed.args[last_idx] = (arg_start, arg_end);
        }
        parsed.data.truncate(arg_end as usize);
        Ok(parsed)
    }
}

const IRCMSG_MAX_ARGS: usize = 20;

// RFC 2812 2.3 Messages
//
// Each IRC message may consist of up to three main parts:
// the prefix (optional), the command, and the command
// parameters (of which there may be up to 15). The prefix,
// command, and all parameters are separated by one (or
// more) ASCII space character(s) (0x20).

#[derive(Clone, Debug)]
pub struct IrcMsg {
    data: Vec<u8>,
    prefix: (u32, u32),
    command: (u32, u32),
    arg_len: u32,
    args: [(u32, u32); IRCMSG_MAX_ARGS],
}

impl IrcMsg {
    pub fn new(data: Vec<u8>) -> Result<IrcMsg, ParseError> {
        let parsed = match IrcParser::parse(data) {
            Ok(parsed) => parsed,
            Err(err) => return Err(err)
        };
        if !::std::str::from_utf8(parsed.get_prefix_raw()).is_ok() {
            return Err(ParseError::EncodingError)
        }
        if !::std::str::from_utf8(parsed.get_command_raw()).is_ok() {
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
        &self.data[prefix_start as usize..prefix_end as usize]
    }

    pub fn get_prefix_str(&self) -> &str {
        unsafe { ::std::str::from_utf8_unchecked(self.get_prefix_raw()) }
    }

    pub fn get_prefix<'a>(&'a self) -> IrcMsgPrefix<'a> {
        IrcMsgPrefix::new(self.get_prefix_str().into_cow())
    }

    fn get_command_raw<'a>(&'a self) -> &[u8] {
        let (command_start, command_end) = self.command;
        &self.data[command_start as usize..command_end as usize]
    }

    pub fn get_command(&self) -> &str {
        unsafe { ::std::str::from_utf8_unchecked(self.get_command_raw()) }
    }

    pub fn get_args(&self) -> Vec<&[u8]> {
        let mut out = Vec::with_capacity(self.arg_len as usize);
        for i in 0..(self.arg_len as usize) {
            let (arg_start, arg_end) = self.args[i];
            out.push(&self.data[arg_start as usize..arg_end as usize]);
        }
        out
    }

    pub fn len(&self) -> usize {
        self.arg_len as usize
    }

    pub fn into_bytes(self) -> Vec<u8> {
        self.data
    }

    pub fn as_bytes(&self) -> &[u8] {
        self.data.as_ref()
    }
}

impl Index<usize> for IrcMsg {
    type Output = [u8];

    fn index<'a>(&'a self, index: usize) -> &'a [u8] {
        let (arg_start, arg_end) = self.args[index];
        &self.data[arg_start as usize..arg_end as usize]
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
                Err(err) => panic!("err: {:?}", err)
            };
            assert!(parsed.has_prefix());
            assert_eq!(parsed.get_prefix_raw(), b"prefix");
            assert_eq!(parsed.get_prefix_str(), "prefix");
            assert_eq!(parsed.get_command(), "PING");
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
                Err(err) => panic!("err: {:?}", err)
            };
            assert!(parsed.has_prefix());
            assert_eq!(parsed.get_prefix_raw(), b"prefix");
            assert_eq!(parsed.get_prefix_str(), "prefix");
            assert_eq!(parsed.get_command(), "PING");
            assert_eq!(parsed.get_args().as_slice(), [b"foo", b"bar baz"].as_slice());
        }
    }

    #[test]
    fn test_security() {
        let example: Vec<_> = b":prefix PING foo\r\n:prefix2 PING bar\r\n".iter().map(|&x| x).collect();
        let safe = match IrcParser::parse(example) {
            Ok(parsed) => parsed.into_bytes(),
            Err(err) => panic!("Should have been able to parse. err: {:?}", err)
        };
        assert_eq!(safe.as_slice(), b":prefix PING foo");
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


#[derive(Clone)]
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

    #[allow(dead_code)]
    pub fn apply<'a>(&self, prefix: Cow<'a, str>) -> IrcMsgPrefix<'a> {
        IrcMsgPrefix {
            data: prefix,
            slicer: self.clone()
        }
    }
}

/// An IRC prefix, which identifies the source of a message.
#[derive(Clone)]
pub struct IrcMsgPrefix<'a> {
    data: Cow<'a, str>,
    slicer: PrefixSlicer
}

impl<'a> IrcMsgPrefix<'a> {
    /// Parse a Cow<'_, str> into a IrcMsgPrefix
    pub fn new(s: Cow<'a, str>) -> IrcMsgPrefix {
        let slicer = PrefixSlicer::new(&s);
        IrcMsgPrefix {
            data: s,
            slicer: slicer
        }
    }

    /// The nick component of a prefix
    pub fn nick(&'a self) -> Option<&'a str> {
        self.slicer.nick_idx_pair.slice_on(&self.data)
    }

    /// The username component of a prefix
    pub fn username(&'a self) -> Option<&'a str> {
        self.slicer.username_idx_pair.slice_on(&self.data)
    }

    /// The hostname component of a prefix
    pub fn hostname(&'a self) -> &'a str {
        self.slicer.hostname_idx_pair.slice_on(&self.data)
    }

    /// Get the protocol representation as a slice
    pub fn as_slice(&self) -> &str {
        &self.data
    }

    /// Get an owned copy
    pub fn to_owned(&self) -> IrcMsgPrefix<'static> {
        IrcMsgPrefix {
            data: self.data.to_string().into_cow(),
            slicer: self.slicer.clone()
        }
    }

    /// Get an owned copy with a replaced nick
    pub fn with_nick(&self, nick: &str) -> Option<IrcMsgPrefix<'static>> {
        match (self.nick(), self.username(), self.hostname()) {
            (Some(_), Some(username), hostname) => {
                let prefix_data = format!("{}!{}@{}", nick, username, hostname);
                Some(IrcMsgPrefix::new(prefix_data.into_cow()))
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

impl<'a> fmt::Debug for IrcMsgPrefix<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "IrcMsgPrefix::new({})", self.as_slice())
    }
}
