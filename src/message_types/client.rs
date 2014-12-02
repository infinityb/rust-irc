use parse::IrcMsg;


macro_rules! msg_wrapper_common {
	($t:ident) => {
		impl $t {
			pub fn into_bytes(self) -> Vec<u8> {
				self.into_irc_msg().into_bytes()
			}

			pub fn into_irc_msg(self) -> IrcMsg {
				let $t(msg) = self;
				msg
			}

			pub fn to_irc_msg<'a>(&'a self) -> &'a IrcMsg {
				let $t(ref msg) = *self;
				msg
			}
		}
	}
}


#[deriving(Clone, Show)]
pub enum OutgoingMsg {
	Pong(Pong),
}


#[deriving(Clone, Show)]
pub struct Pong(IrcMsg);
msg_wrapper_common!(Pong)

impl Pong {
	pub fn new(argument: &str) -> Pong {
		let mut msg = Vec::with_capacity(5 + argument.as_bytes().len());
		msg.push_all(b"PONG ");
		msg.push_all(argument.as_bytes());
		Pong(IrcMsg::new(msg).ok().expect("Generated invalid message"))
	}
}


#[deriving(Clone, Show)]
pub struct Privmsg(IrcMsg);
msg_wrapper_common!(Privmsg)

impl Privmsg {
	pub fn new(target: &str, argument: &[u8]) -> Privmsg {
		let mut msg = Vec::new();
		msg.push_all(b"PRIVMSG ");
		msg.push_all(target.as_bytes());
		msg.push_all(b" :");
		msg.push_all(argument);

		Privmsg(IrcMsg::new(msg).ok().expect("Generated invalid message"))
	}

	pub fn new_ctcp(target: &str, argument: &[u8]) -> Privmsg {
		let mut msg = Vec::new();
		msg.push_all(b"PRIVMSG ");
		msg.push_all(target.as_bytes());
		msg.push_all(b" \x01");
		msg.push_all(argument);
		msg.push_all(b"\x01");
		Privmsg(IrcMsg::new(msg).ok().expect("Generated invalid message"))
	}
}
