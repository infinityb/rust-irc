
pub use self::parse::IrcMsg as IrcMsgNew;
pub use self::parse::{
	can_target_channel,
	is_channel,
};

pub use self::oldparse::IrcMsg as IrcMsg;
pub use self::parse::IrcMsgPrefix;
pub use self::parse::is_full_prefix;


mod parse;
mod oldparse;