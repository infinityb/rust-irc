
pub use self::parse::IrcMsg;

pub use self::parse::{
    can_target_channel,
    is_channel,
};

pub use self::parse::IrcMsgPrefix;
pub use self::parse::is_full_prefix;

mod parse;
pub mod parse2;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ParseErrorKind {
    EncodingError,
    Truncated,
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

