use std::mem;
use std::ops;
use std::borrow::{Borrow, BorrowMut, ToOwned};

use super::{ParseErrorKind, ParseError};
use ::slice::Slice;
use ::parse_helpers;
use ::parse::IrcMsg as IrcMsgLegacy;


#[derive(Clone)]
pub struct IrcMsgBuf {
    inner: Vec<u8>,
}

#[derive(Debug)]
pub struct IrcMsg {
    inner: Slice,
}

pub struct IrcMsgPrefix {
    inner: Slice,
}

impl ops::Deref for IrcMsgBuf {
    type Target = IrcMsg;

    fn deref<'a>(&'a self) -> &'a IrcMsg {
        self.as_irc_msg()
    }
}

impl Borrow<IrcMsg> for IrcMsgBuf {
    fn borrow(&self) -> &IrcMsg {
        self.as_irc_msg()
    }
}

impl AsRef<IrcMsg> for IrcMsgBuf {
    fn as_ref(&self) -> &IrcMsg {
        self.as_irc_msg()
    }
}

impl BorrowMut<IrcMsg> for IrcMsgBuf {
    fn borrow_mut(&mut self) -> &mut IrcMsg {
        self.as_irc_msg_mut()
    }
}

impl AsMut<IrcMsg> for IrcMsgBuf {
    fn as_mut(&mut self) -> &mut IrcMsg {
        self.as_irc_msg_mut()
    }
}

impl ToOwned for IrcMsg {
    type Owned = IrcMsgBuf;

    fn to_owned(&self) -> IrcMsgBuf {
        IrcMsgBuf { inner: self.inner.to_owned() }
    }
}

impl IrcMsgBuf {
    pub fn new(mut buf: Vec<u8>) -> Result<IrcMsgBuf, ParseError> {
        let msg_len = try!(IrcMsg::new(&buf)).as_bytes().len();
        buf.truncate(msg_len);
        Ok(IrcMsgBuf { inner: buf })
    }

    pub fn into_inner(self) -> Vec<u8> {
        self.inner
    }

    fn as_irc_msg(&self) -> &IrcMsg {
        unsafe { IrcMsg::from_u8_slice_unchecked(&self.inner) }
    }

    fn as_irc_msg_mut(&mut self) -> &mut IrcMsg {
        unsafe { IrcMsg::from_u8_slice_unchecked_mut(&mut self.inner) }
    }

    pub fn from_legacy(legacy: IrcMsgLegacy) -> IrcMsgBuf {
        IrcMsgBuf::new(legacy.into_bytes()).unwrap()
    }

    pub fn into_legacy(self) -> IrcMsgLegacy {
        IrcMsgLegacy::new(self.inner).unwrap()
    }
}

impl IrcMsg {
    pub fn new(buf: &[u8]) -> Result<&IrcMsg, ParseError>  {
        let buf = parse_helpers::first_line(buf);
        try!(IrcMsg::validate_buffer(&buf));

        Ok(unsafe {
            // Invariant is maintained by IrcMsg::validate_buffer
            IrcMsg::from_u8_slice_unchecked(buf)
        })
    }

    pub fn from_legacy(legacy: &IrcMsgLegacy) -> &IrcMsg {
        IrcMsg::new(legacy.as_bytes()).unwrap()
    }

    fn validate_buffer(buf: &[u8]) -> Result<(), ParseError> {
        let mut parser = IrcParser::new();

        for &byte in buf.iter()  {
            parser = match parser.push_byte(byte) {
                Ok(new_parser) => new_parser,
                Err(err) => return Err(err.replace_message(buf))
            };
        }

        if let Err(err) = parser.finish() {
            return Err(err.replace_message(buf));
        }

        Ok(())
    }

    /// The following function allows unchecked construction of a irc message
    /// from a u8 slice.  This is unsafe because it does not maintain
    /// the IrcMsg invariant.
    pub unsafe fn from_u8_slice_unchecked(s: &[u8]) -> &IrcMsg {
        mem::transmute(s)
    }

    /// The following function allows unchecked construction of a
    /// mutable irc message from a mutable u8 slice.  This is unsafe because it
    /// does not maintain the IrcMsg invariant.
    pub unsafe fn from_u8_slice_unchecked_mut(s: &mut [u8]) -> &mut IrcMsg {
        mem::transmute(s)
    }

    pub fn as_bytes(&self) -> &[u8] {
        &self.inner
    }

    /// Mutably borrow the underlying storage.  This is private because it
    /// does not maintain the IrcMsg invariant.
    fn as_u8_slice_mut(&mut self) -> &mut [u8] {
        unsafe { mem::transmute(self) }
    }

    pub fn get_prefix<'a>(&'a self) -> Option<&'a IrcMsgPrefix> {
        let buffer = &self.inner[..];
        let (prefix, _) = parse_helpers::split_prefix(buffer);

        if prefix.len() > 0 {
            assert!(prefix.len() > 1);
            Some(IrcMsgPrefix::from_u8_slice_unchecked(&prefix[1..]))
        } else {
            None
        }
    }

    pub fn get_command(&self) -> &str {
        let buffer = &self.inner[..];
        let (_, buffer) = parse_helpers::split_prefix(buffer);
        let (command, _) = parse_helpers::split_command(buffer);

        unsafe { ::std::str::from_utf8_unchecked(command) }
    }

    pub fn tags(&self) -> TagIter {
        let _buffer = &self.inner[..];
        unimplemented!();
        TagIter { arg_body: _buffer }
    }

    pub fn args(&self) -> ArgumentIter {
        let buffer = &self.inner[..];
        let (_, buffer) = parse_helpers::split_prefix(buffer);
        let (_, buffer) = parse_helpers::split_command(buffer);
        ArgumentIter { arg_body: buffer }
    }
}

impl IrcMsgPrefix {
    /// The following function allows unchecked construction of a ogg track
    /// from a u8 slice.  This is private because it does not maintain
    /// the IrcMsgPrefix invariant.
    fn from_u8_slice_unchecked(s: &[u8]) -> &IrcMsgPrefix {
        unsafe { mem::transmute(s) }
    }

    pub fn as_bytes(&self) -> &[u8] {
        &self.inner
    }
}

pub struct TagIter<'a> {
    arg_body: &'a [u8],
}

pub struct ArgumentIter<'a> {
    arg_body: &'a [u8],
}

impl<'a> Iterator for ArgumentIter<'a> {
    type Item = &'a [u8];

    fn next(&mut self) -> Option<&'a [u8]> {
        if self.arg_body.len() == 0 {
            return None;
        }

        let (output, remainder) = parse_helpers::split_arg(self.arg_body);
        self.arg_body = remainder;
        Some(output)
    }
}

#[derive(Clone)]
struct IrcParser(IrcParserState);

#[derive(Copy, Clone)]
enum IrcParserState {
    Initial,
    Prefix,
    CommandStart,
    Command,
    ArgStart,
    Arg,
    ArgEnd,
    RestArg,
}

impl IrcParser {
    fn new() -> IrcParser {
        IrcParser(IrcParserState::Initial)
    }

    fn push_byte(&self, byte: u8) -> Result<IrcParser, ParseError> {
        use self::IrcParserState::*;
        use parse_helpers::is_valid_prefix_byte;

        match (self.0, byte) {
            (Initial, b' ') => Ok(IrcParser(Initial)),
            (Initial, b':') => Ok(IrcParser(Prefix)),
            (Initial, _byte) => Ok(IrcParser(Command)),

            (Prefix, b' ') => Ok(IrcParser(CommandStart)),
            (Prefix, byte) if is_valid_prefix_byte(byte) => Ok(IrcParser(Prefix)),
            (Prefix, _byte) => {
                Err(ParseError::unexpected_byte(byte, "prefix"))
            },

            (CommandStart, b' ') => Ok(IrcParser(CommandStart)),
            (CommandStart, _byte) => Ok(IrcParser(Command)),

            (Command, b' ') => Ok(IrcParser(ArgStart)),
            (Command, _byte) => Ok(IrcParser(Command)),

            (ArgStart, b' ') => Ok(IrcParser(ArgStart)),
            (ArgStart, _byte) => Ok(IrcParser(Arg)),

            (Arg, b' ') => Ok(IrcParser(ArgEnd)),
            (Arg, _byte) => Ok(IrcParser(Arg)),

            (ArgEnd, b' ') => Ok(IrcParser(ArgEnd)),
            (ArgEnd, b':') => Ok(IrcParser(RestArg)),
            (ArgEnd, _byte) => Ok(IrcParser(Arg)),

            (RestArg, _byte) => Ok(IrcParser(RestArg)),
        }
    }

    fn finish(&self) -> Result<(), ParseError> {
        use self::IrcParserState::*;

        let truncated = Err(ParseError::new(ParseErrorKind::Truncated, Vec::new()));
        match self.0 {
            Initial => truncated,
            Prefix => truncated,
            CommandStart => truncated,
            Command => truncated,
            ArgStart => truncated,
            Arg => Ok(()),
            ArgEnd => Ok(()),
            RestArg => Ok(())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::IrcMsg;

    #[test]
    fn test_many_modes() {
        let buf: &[u8] = b":InfinityB!q@d0-0-0-0.abhsia.telus.net MODE # +vvvvvvvvvvvvvvvvvvvv a b c d e f g h i j k l m n o p q r s t";
        let _ = IrcMsg::new(buf).unwrap();
    }

    #[test]
    fn test_many_modes2() {
        let buf: &[u8] = b":InfinityB!q@d0-0-0-0.abhsia.telus.net MODE # +vvvvvvvvvvvvvvvvvvvv a b c d e f g h i j k l m n o p q r s t";
        let msg = IrcMsg::new(buf).unwrap();

        assert_eq!(msg.get_prefix().unwrap().as_bytes(), b"InfinityB!q@d0-0-0-0.abhsia.telus.net" as &[u8]);
        assert_eq!(msg.get_command(), "MODE");

        let mut arg_iter = msg.args();
        assert_eq!(arg_iter.next().unwrap(), b"#");
        assert_eq!(arg_iter.next().unwrap(), b"+vvvvvvvvvvvvvvvvvvvv");
        assert_eq!(arg_iter.next().unwrap(), b"a");
        assert_eq!(arg_iter.next().unwrap(), b"b");
        assert_eq!(arg_iter.next().unwrap(), b"c");
        assert_eq!(arg_iter.next().unwrap(), b"d");
        assert_eq!(arg_iter.next().unwrap(), b"e");
        assert_eq!(arg_iter.next().unwrap(), b"f");
        assert_eq!(arg_iter.next().unwrap(), b"g");
        assert_eq!(arg_iter.next().unwrap(), b"h");
        assert_eq!(arg_iter.next().unwrap(), b"i");
        assert_eq!(arg_iter.next().unwrap(), b"j");
        assert_eq!(arg_iter.next().unwrap(), b"k");
        assert_eq!(arg_iter.next().unwrap(), b"l");
        assert_eq!(arg_iter.next().unwrap(), b"m");
        assert_eq!(arg_iter.next().unwrap(), b"n");
        assert_eq!(arg_iter.next().unwrap(), b"o");
        assert_eq!(arg_iter.next().unwrap(), b"p");
        assert_eq!(arg_iter.next().unwrap(), b"q");
        assert_eq!(arg_iter.next().unwrap(), b"r");
        assert_eq!(arg_iter.next().unwrap(), b"s");
        assert_eq!(arg_iter.next().unwrap(), b"t");
        assert!(arg_iter.next().is_none());
    }
}
