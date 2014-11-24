
pub use self::parse::IrcMsg as IrcMsgNew;
pub use self::parse::{
	can_target_channel,
	is_channel,
};

pub use self::oldparse::IrcMsg as IrcMsg;
pub use self::parse::IrcMsgPrefix;

mod parse;
mod oldparse;