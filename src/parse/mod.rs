use std::error::Error;

pub use self::parse::{IrcMsg, IrcMsgBuf};

// pub use self::parse::{
//     can_target_channel,
//     is_channel,
// };

pub use self::parse::IrcMsgPrefix;

pub mod old_parse;
pub mod parse;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ParseErrorKind {
    EncodingError,
    Truncated,
    // TODO: going away?
    TooManyArguments,
    UnexpectedByte,
    // ...
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ParseError {
    pub kind: ParseErrorKind,
    pub message: Vec<u8>,
    pub error_msg: String,
}

impl ParseError {
    fn new(ekind: ParseErrorKind, msg: Vec<u8>) -> ParseError {
        ParseError {
            kind: ekind,
            message: msg,
            error_msg: "".to_string(),
        }
    }

    fn unexpected_byte(byte: u8, phase: &str) -> ParseError {
        ParseError {
            kind: ParseErrorKind::UnexpectedByte,
            message: Vec::new(),
            error_msg: format!("Unexpected byte `{:?}' in {}", byte, phase)
        }
    }

    fn replace_message(&self, buf: &[u8]) -> ParseError {
        ParseError {
            kind: self.kind.clone(),
            message: buf.to_vec(),
            error_msg: self.error_msg.clone(),
        }
    }
}

impl ::std::fmt::Display for ParseError {
    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        if self.error_msg.len() > 0 {
            write!(f, "ParseError({:?}): {} for {:?}", self.kind, self.error_msg, self.message)
        } else {
            write!(f, "ParseError({:?}) for {:?}", self.kind, self.message)
        }
    }
}

impl Error for ParseError {
    fn description(&self) -> &str {
        if self.error_msg.len() > 0 {
            return &self.error_msg;
        }

        match self.kind {
            ParseErrorKind::EncodingError => "encoding error",
            ParseErrorKind::Truncated => "truncated message",
            ParseErrorKind::TooManyArguments => "too many arguments",
            ParseErrorKind::UnexpectedByte => "unexpected byte",
        }
    }
}
