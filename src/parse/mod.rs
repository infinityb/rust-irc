
pub use self::parse::IrcMsg;
pub use self::parse::ParseError;

pub use self::parse::{
    can_target_channel,
    is_channel,
};

pub use self::parse::IrcMsgPrefix;
pub use self::parse::is_full_prefix;

mod parse;
