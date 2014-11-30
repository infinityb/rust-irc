use std::str;

use parse::is_full_prefix;
use parse::IrcMsg;
use irccase::IrcAsciiExt;
use message_types::traits::FromIrcMsg;


pub trait IntoIncomingMsg {
	fn into_incoming_msg(self: Self) -> IncomingMsg;
}

fn to_incoming<T: IntoIncomingMsg + FromIrcMsg>(msg: IrcMsg) -> IncomingMsg {
	let intermed: Result<T, IrcMsg> = FromIrcMsg::from_irc_msg(msg);
	match intermed {
		Ok(typed_msg) => typed_msg.into_incoming_msg(),
		Err(msg) => IncomingMsg::Unknown(msg)
	}
}

macro_rules! impl_into_incoming_msg {
    ($id:ident) => {
        impl IntoIncomingMsg for $id {
            fn into_incoming_msg(self) -> IncomingMsg {
            	IncomingMsg::$id(self)
            }
        }
    }
}

macro_rules! incoming_msg_common {
	($t:ident) => {
		impl $t {
			pub fn unwrap(self) -> IrcMsg {
				let $t(msg) = self;
				msg
			}

			pub fn borrow_inner<'a>(&'a self) -> &'a IrcMsg {
				let $t(ref msg) = *self;
				msg
			}
		}
	}
}

#[deriving(Clone, Show)]
pub enum IncomingMsg {
	Join(Join),
	Ping(Ping),
	Privmsg(Privmsg),
	Quit(Quit),
	Topic(Topic),
	Kick(Kick),
	Nick(Nick),
	Mode(Mode),
	Part(Part),
	Numeric(u16, Numeric),
	Unknown(IrcMsg),
}

impl IncomingMsg {
	pub fn from_msg(msg: IrcMsg) -> IncomingMsg {
		match msg.get_command() {
			"JOIN" => to_incoming::<Join>(msg),
			"PING" => to_incoming::<Ping>(msg),
			"PRIVMSG" => to_incoming::<Privmsg>(msg),
			"QUIT" => to_incoming::<Quit>(msg),
			"TOPIC" => to_incoming::<Topic>(msg),
			"KICK" => to_incoming::<Kick>(msg),
			"NICK" => to_incoming::<Nick>(msg),
			"MODE" => to_incoming::<Mode>(msg),
			"PART" => to_incoming::<Part>(msg),
			_ => match str::from_str::<u16>(msg.get_command()) {
				Some(_) => to_incoming::<Numeric>(msg),
				None => IncomingMsg::Unknown(msg)
			}
		}
	}

	pub fn is_privmsg(&self) -> bool {
		match *self {
			IncomingMsg::Privmsg(_) => true,
			_ => false
		}
	}

	pub fn borrow_inner<'a>(&'a self) -> &'a IrcMsg {
		match *self {
			IncomingMsg::Join(ref msg) => msg.borrow_inner(),
			IncomingMsg::Ping(ref msg) => msg.borrow_inner(),
			IncomingMsg::Privmsg(ref msg) => msg.borrow_inner(),
			IncomingMsg::Quit(ref msg) => msg.borrow_inner(),
			IncomingMsg::Topic(ref msg) => msg.borrow_inner(),
			IncomingMsg::Kick(ref msg) => msg.borrow_inner(),
			IncomingMsg::Nick(ref msg) => msg.borrow_inner(),
			IncomingMsg::Mode(ref msg) => msg.borrow_inner(),
			IncomingMsg::Part(ref msg) => msg.borrow_inner(),
			IncomingMsg::Numeric(_, ref msg) => msg.borrow_inner(),
			IncomingMsg::Unknown(ref msg) => msg,
		}
	}
}

#[test]
fn test_incoming() {
	let mut msg_raw = Vec::new();
	msg_raw.push_all(b":person!user@host JOIN #foo");
	let msg = IrcMsg::new(msg_raw).unwrap();

	match IncomingMsg::from_msg(msg) {
		IncomingMsg::Join(ref join) => {
			assert_eq!(join.get_nick(), "person");
			assert_eq!(join.get_channel(), "#foo");
		},
		_ => panic!("Wrong IncomingMsg enum value")
	}
}


#[deriving(Clone, Show)]
pub struct Join(IrcMsg);
incoming_msg_common!(Join)
impl_into_incoming_msg!(Join)

impl Join {
	pub fn get_channel(&self) -> &str {
		let Join(ref msg) = *self;
		unsafe { str::from_utf8_unchecked(&msg[0]) }
	}

	pub fn get_nick<'a>(&'a self) -> &'a str {
		let Join(ref msg) = *self;
		let prefix = msg.get_prefix_str();
		prefix[..prefix.find('!').unwrap()]
	}
}

impl FromIrcMsg for Join {
	fn from_irc_msg(msg: IrcMsg) -> Result<Join, IrcMsg> {
		if !msg.get_command().eq_ignore_irc_case("JOIN") {
			return Err(msg);
		}
		if msg.len() == 0 {
			warn!("Invalid JOIN: Not enough arguments {}", msg.len());
			return Err(msg);
		}
		if !is_full_prefix(msg.get_prefix_str()) {
			warn!("Invalid JOIN: Insufficient prefix `{}`", msg.get_prefix_str());
			return Err(msg);
		}
		Ok(Join(msg))
	}
}

#[test]
fn test_join_basics() {
	let valid_messages: &[(&[u8], &str, &str)] = &[
		// Standard messages
		(b":person!user@host JOIN #foobar", "person", "#foobar"),
		(b":person!user@host JOIN #foobar\n", "person", "#foobar"),
		(b":person!user@host JOIN #foobar\r\n", "person", "#foobar"),
	];

	for &(raw, nick, channel) in valid_messages.iter() {
		let mut raw_owned = Vec::with_capacity(raw.len());
		raw_owned.push_all(raw);

		let msg = IrcMsg::new(raw_owned).unwrap();
		let join_msg: Join = FromIrcMsg::from_irc_msg(msg).ok().unwrap();
		assert_eq!(join_msg.get_nick(), nick);
		assert_eq!(join_msg.get_channel(), channel);
	}
}


#[deriving(Clone, Show)]
pub struct Numeric(IrcMsg);
incoming_msg_common!(Numeric)

impl Numeric {
	pub fn get_code(&self) -> u16 {
		let Numeric(ref msg) = *self;
		from_str::<u16>(msg.get_command()).unwrap()
	}
}

impl IntoIncomingMsg for Numeric {
	fn into_incoming_msg(self) -> IncomingMsg {
		let numeric_num = self.get_code();
		IncomingMsg::Numeric(numeric_num, self)	
	}
}

impl FromIrcMsg for Numeric {
	fn from_irc_msg(msg: IrcMsg) -> Result<Numeric, IrcMsg>  {
		match from_str::<u16>(msg.get_command()) {
			Some(_) => Ok(Numeric(msg)),
			None => Err(msg)
		}
	}
}


#[deriving(Clone, Show)]
pub struct Quit(IrcMsg);
impl_into_incoming_msg!(Quit)
incoming_msg_common!(Quit)

impl Quit {
	pub fn get_nick<'a>(&'a self) -> &'a str {
		let Quit(ref msg) = *self;
		let prefix = msg.get_prefix_str();
		prefix[..prefix.find('!').unwrap()]
	}

	pub fn get_channel(&self) -> &str {
		let Quit(ref msg) = *self;
		unsafe { str::from_utf8_unchecked(&msg[0]) }
	}

	pub fn get_body_raw<'a>(&'a self) -> &'a [u8] {
		let Quit(ref msg) = *self;
		&msg[1]
	}
}

impl FromIrcMsg for Quit {
	fn from_irc_msg(msg: IrcMsg) -> Result<Quit, IrcMsg> {
		if !msg.get_command().eq_ignore_irc_case("QUIT") {
			return Err(msg);
		}
		if msg.len() < 1 {
			warn!("Invalid QUIT: Not enough arguments {}", msg.len());
			return Err(msg);
		}
		Ok(Quit(msg))
	}
}

#[test]
fn test_quit_basics() {
	let msg = IrcMsg::new(b":person!user@host NOTQUIT :server2".to_vec()).unwrap();
	let ping: Result<Ping, _> = FromIrcMsg::from_irc_msg(msg);
	assert!(ping.is_err());
}


#[deriving(Clone, Show)]
pub struct Ping(IrcMsg);
impl_into_incoming_msg!(Ping)
incoming_msg_common!(Ping)

impl Ping {
	pub fn get_response(&self) -> Vec<u8> {
		let Ping(ref msg) = *self;
		let mut response = Vec::new();
		response.push_all(b"PONG ");
		for args in msg.get_args().into_iter() {
			response.push_all(args);
		}
		response
	}
}

impl FromIrcMsg for Ping {
	fn from_irc_msg(msg: IrcMsg) -> Result<Ping, IrcMsg> {
		if !msg.get_command().eq_ignore_irc_case("PING") {
			return Err(msg);
		}
		if msg.len() < 1 {
			warn!("Invalid PING: Not enough arguments {}", msg.len());
			return Err(msg);
		}
		Ok(Ping(msg))
	}
}

#[test]
fn test_ping_basics() {
	let mut raw_owned = Vec::new();
	raw_owned.push_all(b":person!user@host NOTPING server1 :server2");
	let msg = IrcMsg::new(raw_owned).unwrap();
	let ping: Result<Ping, _> = FromIrcMsg::from_irc_msg(msg);
	assert!(ping.is_err());
}


#[deriving(Clone, Show)]
pub struct Privmsg(IrcMsg);
impl_into_incoming_msg!(Privmsg)
incoming_msg_common!(Privmsg)

impl Privmsg {
	pub fn get_channel(&self) -> &str {
		let Privmsg(ref msg) = *self;
		unsafe { str::from_utf8_unchecked(&msg[0]) }
	}

	pub fn get_nick<'a>(&'a self) -> &'a str {
		let Privmsg(ref msg) = *self;
		let prefix = msg.get_prefix_str();
		prefix[..prefix.find('!').unwrap()]
	}

	pub fn get_body_raw<'a>(&'a self) -> &'a [u8] {
		let Privmsg(ref msg) = *self;
		&msg[1]
	}
}

impl FromIrcMsg for Privmsg {
	fn from_irc_msg(msg: IrcMsg) -> Result<Privmsg, IrcMsg> {
		if !msg.get_command().eq_ignore_irc_case("PRIVMSG") {
			return Err(msg);
		}
		if msg.len() < 2 {
			warn!("Invalid PRIVMSG: Not enough arguments {}", msg.len());
			return Err(msg);
		}
		if !is_full_prefix(msg.get_prefix_str()) {
			warn!("Invalid PRIVMSG: Insufficient prefix `{}`", msg.get_prefix_str());
			return Err(msg);
		}
		if !str::is_utf8(&msg[0]) {
			return Err(msg);
		}
		Ok(Privmsg(msg))
	}
}

#[test]
fn test_privmsg_basics() {
	let mut raw_owned = Vec::new();
	raw_owned.push_all(b":person!user@host NOTPRIVMSG server1 :server2");
	let privmsg: Result<Privmsg, _> = FromIrcMsg::from_irc_msg(IrcMsg::new(raw_owned).unwrap());
	assert!(privmsg.is_err());

	let valid_messages: &[(&[u8], &str, &str, &[u8])] = &[
		// Standard messages
		(b":person!user@host PRIVMSG #foobar :foobarbaz",
			"person", "#foobar", b"foobarbaz"),
		(b":person!user@host PRIVMSG #foobar :foobar\r\n",
			"person", "#foobar", b"foobar"),
		(b":person!user@host PRIVMSG #foobar :foobar\r\n",
			"person", "#foobar", b"foobar"),

		// Invalid UTF-8 in message, but we ignore messages
		(b":person!user@host PRIVMSG #foobar :\xe3\x81",
			"person", "#foobar", b"\xe3\x81"),
	];

	for &(raw, nick, channel, body) in valid_messages.iter() {
		let mut raw_owned = Vec::with_capacity(raw.len());
		raw_owned.push_all(raw);

		let msg = IrcMsg::new(raw_owned).unwrap();
		let priv_msg: Privmsg = FromIrcMsg::from_irc_msg(msg).ok().unwrap();
		assert_eq!(priv_msg.get_nick(), nick);
		assert_eq!(priv_msg.get_channel(), channel);
		assert_eq!(priv_msg.get_body_raw(), body);
	}
}


#[deriving(Clone, Show)]
pub struct Topic(IrcMsg);
impl_into_incoming_msg!(Topic)
incoming_msg_common!(Topic)

impl Topic {
	pub fn get_channel(&self) -> &str {
		let Topic(ref msg) = *self;
		unsafe { str::from_utf8_unchecked(&msg[0]) }
	}

	pub fn get_nick<'a>(&'a self) -> &'a str {
		let Topic(ref msg) = *self;
		let prefix = msg.get_prefix_str();
		prefix[..prefix.find('!').unwrap()]
	}

	pub fn get_body_raw<'a>(&'a self) -> &'a [u8] {
		let Topic(ref msg) = *self;
		&msg[1]
	}
}

impl FromIrcMsg for Topic {
	fn from_irc_msg(msg: IrcMsg) -> Result<Topic, IrcMsg> {
		if !msg.get_command().eq_ignore_irc_case("TOPIC") {
			return Err(msg);
		}
		if msg.len() < 2 {
			warn!("Invalid TOPIC: Not enough arguments {}", msg.len());
			return Err(msg);
		}
		if !is_full_prefix(msg.get_prefix_str()) {
			warn!("Invalid TOPIC: Insufficient prefix `{}`", msg.get_prefix_str());
			return Err(msg);
		}
		if !str::is_utf8(&msg[0]) {
			return Err(msg);
		}
		Ok(Topic(msg))
	}
}


#[deriving(Clone, Show)]
pub struct Kick(IrcMsg);
impl_into_incoming_msg!(Kick)
incoming_msg_common!(Kick)


impl Kick {
	pub fn get_channel(&self) -> &str {
		let Kick(ref msg) = *self;
		unsafe { str::from_utf8_unchecked(&msg[0]) }
	}

	/// nick of the user being kicked
	pub fn get_nicked_nick<'a>(&'a self) -> &'a str {
		let Kick(ref msg) = *self;
		unsafe { str::from_utf8_unchecked(&msg[1]) }
	}

	/// nick of the user doing the kicking
	pub fn get_nick<'a>(&'a self) -> &'a str {
		let Kick(ref msg) = *self;
		let prefix = msg.get_prefix_str();
		prefix[..prefix.find('!').unwrap()]
	}

	pub fn get_body_raw<'a>(&'a self) -> &'a [u8] {
		let Kick(ref msg) = *self;
		&msg[1]
	}
}

impl FromIrcMsg for Kick {
	fn from_irc_msg(msg: IrcMsg) -> Result<Kick, IrcMsg> {
		if !msg.get_command().eq_ignore_irc_case("KICK") {
			return Err(msg);
		}
		if msg.len() < 3 {
			warn!("Invalid KICK: Not enough arguments {}", msg.len());
			return Err(msg);
		}
		if !is_full_prefix(msg.get_prefix_str()) {
			warn!("Invalid KICK: Insufficient prefix `{}`", msg.get_prefix_str());
			return Err(msg);
		}
		// msg[0] is channel, msg[1] is kicked nick
		if !str::is_utf8(&msg[0]) || !str::is_utf8(&msg[1]) {
			return Err(msg);
		}
		Ok(Kick(msg))
	}
}


#[deriving(Clone, Show)]
pub struct Nick(IrcMsg);
impl_into_incoming_msg!(Nick)
incoming_msg_common!(Nick)

impl Nick {
	/// The previous nick of the user
	pub fn get_nick<'a>(&'a self) -> &'a str {
		let Nick(ref msg) = *self;
		let prefix = msg.get_prefix_str();
		prefix[..prefix.find('!').unwrap()]
	}

	/// The new nick of the user
	pub fn get_new_nick<'a>(&'a self) -> &'a str {
		let Nick(ref msg) = *self;
		unsafe { str::from_utf8_unchecked(&msg[0]) }
	}
}

impl FromIrcMsg for Nick {
	fn from_irc_msg(msg: IrcMsg) -> Result<Nick, IrcMsg> {
		if !msg.get_command().eq_ignore_irc_case("NICK") {
			return Err(msg);
		}
		if msg.len() < 1 {
			warn!("Invalid NICK: Not enough arguments {}", msg.len());
			return Err(msg);
		}
		if !is_full_prefix(msg.get_prefix_str()) {
			warn!("Invalid NICK: Insufficient prefix `{}`", msg.get_prefix_str());
			return Err(msg);
		}
		// msg[0] is channel, msg[1] is kicked nick
		if !str::is_utf8(&msg[0]) || !str::is_utf8(&msg[1]) {
			return Err(msg);
		}
		Ok(Nick(msg))
	}
}


#[deriving(Clone, Show)]
pub struct Part(IrcMsg);
impl_into_incoming_msg!(Part)
incoming_msg_common!(Part)

impl Part {
	/// The previous nick of the user
	pub fn get_nick<'a>(&'a self) -> &'a str {
		let Part(ref msg) = *self;
		let prefix = msg.get_prefix_str();
		prefix[..prefix.find('!').unwrap()]
	}

	/// The new nick of the user
	pub fn get_channel<'a>(&'a self) -> &'a str {
		let Part(ref msg) = *self;
		unsafe { str::from_utf8_unchecked(&msg[0]) }
	}
}

impl FromIrcMsg for Part {
	fn from_irc_msg(msg: IrcMsg) -> Result<Part, IrcMsg> {
		if !msg.get_command().eq_ignore_irc_case("PART") {
			return Err(msg);
		}
		if msg.len() < 1 {
			warn!("Invalid PART: Not enough arguments {}", msg.len());
			return Err(msg);
		}
		if !is_full_prefix(msg.get_prefix_str()) {
			warn!("Invalid PART: Insufficient prefix `{}`", msg.get_prefix_str());
			return Err(msg);
		}
		if !str::is_utf8(&msg[0]) {
			return Err(msg);
		}
		Ok(Part(msg))
	}
}


#[deriving(Clone, Show)]
pub struct Mode(IrcMsg);
impl_into_incoming_msg!(Mode)
incoming_msg_common!(Mode)

impl Mode {
	/// The previous nick of the user
	pub fn get_nick<'a>(&'a self) -> &'a str {
		let Mode(ref msg) = *self;
		let prefix = msg.get_prefix_str();
		prefix[..prefix.find('!').unwrap()]
	}

	/// The new nick of the user
	pub fn get_channel<'a>(&'a self) -> &'a str {
		let Mode(ref msg) = *self;
		unsafe { str::from_utf8_unchecked(&msg[0]) }
	}
}

impl FromIrcMsg for Mode {
	fn from_irc_msg(msg: IrcMsg) -> Result<Mode, IrcMsg> {
		if !msg.get_command().eq_ignore_irc_case("MODE") {
			return Err(msg);
		}
		if msg.len() < 1 {
			warn!("Invalid MODE: Not enough arguments {}", msg.len());
			return Err(msg);
		}
		if !is_full_prefix(msg.get_prefix_str()) {
			warn!("Invalid MODE: Insufficient prefix `{}`", msg.get_prefix_str());
			return Err(msg);
		}
		unimplemented!();
	}
}