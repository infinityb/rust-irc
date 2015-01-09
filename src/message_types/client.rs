use parse::IrcMsg;


fn byte_length_sum(items: &[&str]) -> usize {
    use std::iter::AdditiveIterator;
    items.iter().map(|x| x.as_bytes().len()).sum()
}

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


#[derive(Clone, Show)]
pub struct Join(IrcMsg);
msg_wrapper_common!(Join);

impl Join {
    pub fn new(channel: &str) -> Join {
        let output_length = byte_length_sum(&[channel]) + 5;

        let mut msg = Vec::with_capacity(output_length);
        msg.push_all(b"JOIN ");
        msg.push_all(channel.as_bytes());

        Join(IrcMsg::new(msg).ok().expect("Generated invalid message"))
    }

    pub fn new_with_key(channel: &str, key: &str) -> Join {
        let output_length = byte_length_sum(&[channel, key]) + 6;

        let mut msg = Vec::with_capacity(output_length);
        msg.push_all(b"JOIN ");
        msg.push_all(channel.as_bytes());
        msg.push_all(b" ");
        msg.push_all(key.as_bytes());

        Join(IrcMsg::new(msg).ok().expect("Generated invalid message"))
    }
}

#[derive(Clone, Show)]
pub struct Nick(IrcMsg);
msg_wrapper_common!(Nick);

impl Nick {
    pub fn new(argument: &str) -> Nick {
        let mut msg = Vec::with_capacity(5 + argument.as_bytes().len());
        msg.push_all(b"NICK ");
        msg.push_all(argument.as_bytes());
        Nick(IrcMsg::new(msg).ok().expect("Generated invalid message"))
    }
}


#[derive(Clone, Show)]
pub struct Pong(IrcMsg);
msg_wrapper_common!(Pong);

impl Pong {
    pub fn new(argument: &str) -> Pong {
        let mut msg = Vec::with_capacity(5 + argument.as_bytes().len());
        msg.push_all(b"PONG ");
        msg.push_all(argument.as_bytes());
        Pong(IrcMsg::new(msg).ok().expect("Generated invalid message"))
    }
}


#[derive(Clone, Show)]
pub struct Privmsg(IrcMsg);
msg_wrapper_common!(Privmsg);

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


#[derive(Clone, Show)]
pub struct User(IrcMsg);
msg_wrapper_common!(User);

impl User {
    //  <user> <mode> <unused> <realname>
    pub fn new(user: &str, mode: &str, unused: &str, realname: &str) -> User {

        let output_length = byte_length_sum(&[user, mode, unused, realname]) + 8;

        let mut msg = Vec::with_capacity(output_length);
        msg.push_all(b"USER ");
        msg.push_all(mode.as_bytes());
        msg.push_all(b" ");
        msg.push_all(unused.as_bytes());
        msg.push_all(b" :");
        msg.push_all(realname.as_bytes());

        User(IrcMsg::new(msg).ok().expect("Generated invalid message"))
    }
}


#[derive(Clone, Show)]
pub struct Who(IrcMsg);
msg_wrapper_common!(Who);

impl Who {
    pub fn new(target: &str) -> Who {
        let mut msg = Vec::new();
        msg.push_all(b"WHO ");
        msg.push_all(target.as_bytes());

        Who(IrcMsg::new(msg).ok().expect("Generated invalid message"))
    }

    pub fn new_opers(target: &str) -> Who {
        let mut msg = Vec::new();
        msg.push_all(b"WHO ");
        msg.push_all(target.as_bytes());
        msg.push_all(b" o");

        Who(IrcMsg::new(msg).ok().expect("Generated invalid message"))
    }
}
