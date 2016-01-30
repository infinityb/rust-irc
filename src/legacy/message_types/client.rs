use super::super::IrcMsg;


fn byte_length_sum(items: &[&str]) -> usize {
    let mut acc = 0;
    for len in items.iter().map(|x| x.as_bytes().len()) {
        acc += len;
    }
    acc
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


#[derive(Clone, Debug)]
pub struct Join(IrcMsg);
msg_wrapper_common!(Join);

impl Join {
    pub fn new(channel: &str) -> Join {
        let output_length = byte_length_sum(&[channel]) + 5;

        let mut msg = Vec::with_capacity(output_length);
        msg.extend(b"JOIN ");
        msg.extend(channel.as_bytes());

        Join(IrcMsg::new(msg).ok().expect("Generated invalid message"))
    }

    pub fn new_with_key(channel: &str, key: &str) -> Join {
        let output_length = byte_length_sum(&[channel, key]) + 6;

        let mut msg = Vec::with_capacity(output_length);
        msg.extend(b"JOIN ");
        msg.extend(channel.as_bytes());
        msg.extend(b" ");
        msg.extend(key.as_bytes());

        Join(IrcMsg::new(msg).ok().expect("Generated invalid message"))
    }
}

#[derive(Clone, Debug)]
pub struct Nick(IrcMsg);
msg_wrapper_common!(Nick);

impl Nick {
    pub fn new(argument: &str) -> Nick {
        let mut msg = Vec::with_capacity(5 + argument.as_bytes().len());
        msg.extend(b"NICK ");
        msg.extend(argument.as_bytes());
        Nick(IrcMsg::new(msg).ok().expect("Generated invalid message"))
    }
}


#[derive(Clone, Debug)]
pub struct Ping(IrcMsg);
msg_wrapper_common!(Ping);

impl Ping {
    pub fn new(argument: &str) -> Ping {
        let mut msg = Vec::with_capacity(6 + argument.as_bytes().len());
        msg.extend(b"PING :");
        msg.extend(argument.as_bytes());
        Ping(IrcMsg::new(msg).ok().expect("Generated invalid message"))
    }
}

#[derive(Clone, Debug)]
pub struct Pong(IrcMsg);
msg_wrapper_common!(Pong);

impl Pong {
    pub fn new(argument: &str) -> Pong {
        let mut msg = Vec::with_capacity(5 + argument.as_bytes().len());
        msg.extend(b"PONG ");
        msg.extend(argument.as_bytes());
        Pong(IrcMsg::new(msg).ok().expect("Generated invalid message"))
    }
}


#[derive(Clone, Debug)]
pub struct Privmsg(IrcMsg);
msg_wrapper_common!(Privmsg);

impl Privmsg {
    pub fn new(target: &str, argument: &[u8]) -> Privmsg {
        let mut msg = Vec::new();
        msg.extend(b"PRIVMSG ");
        msg.extend(target.as_bytes());
        msg.extend(b" :");
        msg.extend(argument);

        Privmsg(IrcMsg::new(msg).ok().expect("Generated invalid message"))
    }

    pub fn new_ctcp(target: &str, argument: &[u8]) -> Privmsg {
        let mut msg = Vec::new();
        msg.extend(b"PRIVMSG ");
        msg.extend(target.as_bytes());
        msg.extend(b" \x01");
        msg.extend(argument);
        msg.extend(b"\x01");
        Privmsg(IrcMsg::new(msg).ok().expect("Generated invalid message"))
    }
}


#[derive(Clone, Debug)]
pub struct User(IrcMsg);
msg_wrapper_common!(User);

impl User {
    //  <user> <mode> <unused> <realname>
    pub fn new(user: &str, mode: &str, unused: &str, realname: &str) -> User {

        let output_length = byte_length_sum(&[user, mode, unused, realname]) + 8;

        let mut msg = Vec::with_capacity(output_length);
        msg.extend(b"USER ");
        msg.extend(user.as_bytes());
        msg.extend(b" ");
        msg.extend(mode.as_bytes());
        msg.extend(b" ");
        msg.extend(unused.as_bytes());
        msg.extend(b" :");
        msg.extend(realname.as_bytes());

        User(IrcMsg::new(msg).ok().expect("Generated invalid message"))
    }
}


#[derive(Clone, Debug)]
pub struct Who(IrcMsg);
msg_wrapper_common!(Who);

impl Who {
    pub fn new(target: &str) -> Who {
        let mut msg = Vec::new();
        msg.extend(b"WHO ");
        msg.extend(target.as_bytes());

        Who(IrcMsg::new(msg).ok().expect("Generated invalid message"))
    }

    pub fn new_opers(target: &str) -> Who {
        let mut msg = Vec::new();
        msg.extend(b"WHO ");
        msg.extend(target.as_bytes());
        msg.extend(b" o");

        Who(IrcMsg::new(msg).ok().expect("Generated invalid message"))
    }
}

#[derive(Clone, Debug)]
pub struct Quit(IrcMsg);
msg_wrapper_common!(Quit);

impl Quit {
    pub fn new(target: &str) -> Quit {
        let mut msg = Vec::new();
        msg.extend(b"QUIT :");
        msg.extend(target.as_bytes());

        Quit(IrcMsg::new(msg).ok().expect("Generated invalid message"))
    }
}
