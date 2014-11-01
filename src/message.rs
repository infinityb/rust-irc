use std::string::{String};
use std::fmt;
use parse::{
    IrcMsg,
    is_channel,
    can_target_channel
};

pub type IrcProtocolMessage = self::IrcProtocolMessage::IrcProtocolMessage;
#[allow(non_snake_case)]
pub mod IrcProtocolMessage {
    #[deriving(Clone)]
    pub enum IrcProtocolMessage {
        Ping(String, Option<String>),
        Pong(String),
        Notice(String, String),
        Join(String),
        Numeric(u16, Vec<String>),
        // parsed but not processed into a safe message type. command, rest
        Unknown(String, Vec<String>)
    }
}


impl fmt::Show for IrcProtocolMessage {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            IrcProtocolMessage::Ping(ref s1, None) => write!(f, "PING {}", s1),
            IrcProtocolMessage::Ping(ref s1, Some(ref s2)) => write!(f, "PING {} {}", s1, s2),
            IrcProtocolMessage::Pong(ref s1) => write!(f, "PONG {}", s1),
            IrcProtocolMessage::Notice(ref s1, ref s2) => {
                write!(f, "NOTICE {} :{}", s1, s2)
            },
            _ => write!(f, "WHAT")
        }
    }
}


#[deriving(PartialEq, Clone)]
pub struct IrcHostmask {
    nick: String,
    user: String,
    host: String
}

impl fmt::Show for IrcHostmask {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}!{}@{}",
            self.nick.as_slice(),
            self.user.as_slice(),
            self.host.as_slice())
    }
}


#[deriving(PartialEq, Clone, Show)]
pub enum IrcPrefix {
    IrcHostmaskPrefix(IrcHostmask),
    IrcOtherPrefix(String)
}

impl IrcPrefix {
    pub fn new(text: &str) -> IrcPrefix {
        let parts: Vec<&str> = text.splitn(1, '!').collect();
        let (nick, rest) = match parts.as_slice() {
            [_] => return IrcOtherPrefix(String::from_str(text)),
            [nick, rest] => (nick, rest),
            _ => panic!("programmer error")
        };

        let parts: Vec<&str> = rest.splitn(1, '@').collect();
        let (user, host) = match parts.as_slice() {
            [_] => return IrcOtherPrefix(String::from_str(text)),
            [user, rest] => (user, rest),
            _ => panic!("programmer error")
        };  
        IrcHostmaskPrefix(IrcHostmask {
            nick: String::from_str(nick),
            user: String::from_str(user),
            host: String::from_str(host),
        })
    }

    pub fn with_nick(&self, nick: &str) -> IrcPrefix {
        match *self {
            IrcHostmaskPrefix(ref hostmask) => IrcHostmaskPrefix(IrcHostmask {
                nick: nick.to_string(),
                user: hostmask.user.clone(),
                host: hostmask.host.clone(),
            }),
            IrcOtherPrefix(_) => IrcOtherPrefix(nick.to_string())
        }
    }

    pub fn to_string(&self) -> String {
        match *self {
            IrcHostmaskPrefix(ref hostmask) => format!("{}", hostmask),
            IrcOtherPrefix(ref prefix) => prefix.clone()
        }
    }
}


#[deriving(Clone)]
pub struct IrcMessage {
    msg: Option<IrcMsg<'static>>,
    prefix: Option<IrcPrefix>,
    prefix_raw: Option<String>,
    message: IrcProtocolMessage,
    command: String,
    args: Vec<String>
}

fn parse_message_args(text: &str) -> Result<Vec<String>, Option<String>> {
    if text.len() == 0 {
        return Err(from_str("Invalid IRC message"));
    }
    if text.char_at(0) == ':' {
        return Ok(vec![String::from_str(text.slice_from(1))]);
    }

    let (arg_parts, rest) = match text.find_str(" :") {
        Some(val) => (text.slice_to(val), Some(text.slice_from(val + 2))),
        None => (text, None)
    };

    let mut output: Vec<String> = arg_parts.split(' ')
            .map(|s| String::from_str(s)).collect();

    if rest.is_some() {
        output.push(String::from_str(rest.unwrap()));
    }
    Ok(output)
}


fn parse_message_rest(text: &str) -> Result<(String, Vec<String>), Option<String>> {
    let parts: Vec<&str> = text.splitn(1, ' ').collect();
    let args = match parse_message_args(parts[1]) {
        Ok(args) => args,
        Err(err) => return Err(err)
    };
    Ok((String::from_str(parts[0]), args))
}


impl IrcMessage {
    pub fn notice(destination: &str, message: &str) -> IrcMessage {
        let mut tmp = IrcMessage {
            msg: None,
            prefix: None,
            prefix_raw: None,
            message: IrcProtocolMessage::Notice(
                destination.to_string(), message.to_string()),
            command: "NOTICE".to_string(),
            args: vec![
                destination.to_string(),
                message.to_string()
            ]
        };
        tmp.msg = IrcMsg::new(tmp.to_irc().into_maybe_owned());
        tmp
    }

    pub fn from_str(text: &str) -> Result<IrcMessage, String> {
        if text.len() == 0 {
            return Err("Invalid IRC message; empty".to_string());
        }
        
        let (prefix, command, mut args) = if text.char_at(0) == ':' {
                let parts: Vec<&str> = text.splitn(1, ' ').collect();
                if parts.len() < 2 {
                    return Err("Invalid IRC message".to_string());
                }
                let (command, args) = match parse_message_rest(parts[1]) {
                    Ok(result) => result,
                    Err(err) => return Err(format!("Invalid IRC message: {}", err))
                };

                (Some(String::from_str(parts[0].slice_from(1))), command, args)
            } else {
                assert!(text.len() > 0);
                let (command, args) = match parse_message_rest(text) {
                    Ok(result) => result,
                    Err(err) => return Err(format!("Invalid IRC message: {}", err))
                };
                (None, command, args)
            };

        let message_command = command.clone();
        let message_args = args.clone();

        let message = match (command.as_slice(), args.len()) {
            ("PING", 1...2) => {
                IrcProtocolMessage::Ping(args.remove(0).unwrap(), args.remove(0))
            },
            ("PING", _) => return Err(
                "Invalid IRC message: too many arguments to PING".to_string()),
            ("PONG", 1) => IrcProtocolMessage::Pong(args.remove(0).unwrap()),
            ("PONG", _) => return Err(
                "Invalid IRC message: too many arguments to PONG".to_string()),
            (_, _) => {
                match from_str(command.as_slice()) {
                    Some(num) => IrcProtocolMessage::Numeric(num, args),
                    None => IrcProtocolMessage::Unknown(command, args)
                }
            }
        };

        let prefix_parsed = match prefix {
            Some(ref pref) => Some(IrcPrefix::new(pref.as_slice())),
            None => None
        };

        let msg = match IrcMsg::new(text.to_string().into_maybe_owned()) {
            Some(msg) => msg,
            None => return Err("Invalid IRC message; parse failure".to_string())
        };

        Ok(IrcMessage {
            msg: Some(msg),
            prefix: prefix_parsed,
            prefix_raw: prefix,
            message: message,
            command: message_command,
            args: message_args
        })
    }

    pub fn to_irc(&self) -> String {
        match self.message {
            IrcProtocolMessage::Notice(ref dest, ref data) => {
                format!("NOTICE {} :{}", dest[], data[])
            }
            _ => unimplemented!()
        }
    }

    #[inline]
    pub fn is_privmsg(&self) -> bool {
        self.command() == "PRIVMSG"
    }

    #[inline]
    pub fn target_is_channel(&self) -> bool {
        self.channel().is_some()
    }

    // can_target_channel is incomplete
    // An enum of target types is probably better here, instead of a Option<&str>
    pub fn channel(&self) -> Option<&str> {
        if can_target_channel(self.command()) && self.get_args().len() > 0 {
            let channel_name = self.get_args()[0];
            if is_channel(channel_name) {
                return Some(channel_name)
            }
        }
        None
    }

    #[inline]
    pub fn source_nick<'a>(&'a self) -> Option<&'a str> {
        match self.msg {
            Some(ref msg) => msg.source_nick(),
            None => None,
        }
    }

    pub fn get_prefix<'a>(&'a self) -> Option<&'a IrcPrefix> {
        match self.prefix {
            Some(ref pref) => Some(pref),
            None => None
        }
    }

    pub fn get_prefix_raw<'a>(&'a self) -> Option<&'a str> {
        match self.prefix_raw {
            Some(ref prefix) => Some(prefix.as_slice()),
            None => None
        }
    }

    pub fn get_message<'a>(&'a self) -> &'a IrcProtocolMessage {
        &self.message
    }

    #[inline]
    pub fn command<'a>(&'a self) -> &'a str {
        self.msg.as_ref().unwrap().get_command()
    }

    #[inline]
    pub fn get_args<'a>(&'a self) -> Vec<&'a str> {
        self.msg.as_ref().unwrap().get_args()
    }
}


impl fmt::Show for IrcMessage {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut arg_string = String::new();
        arg_string.push_str("[");
        for part in self.args.iter().map(|s| s.as_slice()) {
            arg_string.push_str(format!("{}, ", part.as_slice()).as_slice());
        }
        arg_string.push_str("]");

        match self.prefix_raw {
            Some(ref prefix) => write!(f, "IrcMessage({}, {}, {})",
                prefix.as_slice(), self.command.as_slice(), arg_string.as_slice()),
            None => write!(f, "IrcMessage({}, {})",
                self.command.as_slice(), arg_string.as_slice())
        }
    }
}


#[test]
fn test_irc_message_general() {
    match IrcMessage::from_str("") {
        Ok(_) => {
            panic!("empty string is invalid")
        },
        Err(_) => ()
    };

    match IrcMessage::from_str(":") {
        Ok(_) => {
            panic!("single colon is invalid")
        },
        Err(_) => ()
    };

    match IrcMessage::from_str(" ") {
        Ok(_) => {
            panic!("single space is invalid")
        },
        Err(_) => ()
    };

    match IrcMessage::from_str(":nick!user@host PING server1 :server2") {
        Ok(message) => {
            match message.prefix {
                Some(IrcHostmaskPrefix(ref data)) => {
                    assert_eq!(data.nick.as_slice(), "nick");
                    assert_eq!(data.user.as_slice(), "user");
                    assert_eq!(data.host.as_slice(), "host");
                }
                _ => panic!("invalid parsed prefix")
            };
        },
        Err(_) => panic!("failed to parse")
    };

    match IrcMessage::from_str("PING server1") {
        Ok(message) => {
            assert_eq!(message.prefix_raw, None);
            assert_eq!(message.command.as_slice(), "PING");
            assert_eq!(message.args.len(), 1);
        },
        Err(_) => panic!("failed to parse")
    };


    match IrcMessage::from_str("PING server1 server2") {
        Ok(message) => {
            assert_eq!(message.prefix_raw, None);
            assert_eq!(message.command.as_slice(), "PING");
            assert_eq!(message.args.len(), 2);
        },
        Err(_) => panic!("failed to parse")
    };

    match IrcMessage::from_str("PING server1 server2 server3") {
        Ok(_) => panic!("should fail to parse"),
        Err(_) => ()
    };

    match IrcMessage::from_str(":somewhere PING server1") {
        Ok(message) => {
            assert_eq!(message.prefix_raw, Some(String::from_str("somewhere")));
            assert_eq!(message.command.as_slice(), "PING");
            assert_eq!(message.args.len(), 1);
        },
        Err(_) => panic!("failed to parse")
    };
    
    match IrcMessage::from_str(":somewhere PING server1 server2") {
        Ok(message) => {
            assert_eq!(message.prefix_raw, Some(String::from_str("somewhere")));
            assert_eq!(message.command.as_slice(), "PING");
            assert_eq!(message.args.len(), 2);
            assert_eq!(message.args[0].as_slice(), "server1");
            assert_eq!(message.args[1].as_slice(), "server2");
            match message.message {
                IrcProtocolMessage::Ping(s1, s2) => {
                    assert_eq!(s1, String::from_str("server1"));
                    assert_eq!(s2, Some(String::from_str("server2")));
                },
                _ => assert!(false)
            };

        },
        Err(_) => panic!("failed to parse")
    };

    match IrcMessage::from_str(":somewhere PING server1 :server2") {
        Ok(message) => {
            assert_eq!(message.prefix, Some(IrcOtherPrefix(String::from_str("somewhere"))));
            assert_eq!(message.prefix_raw, Some(String::from_str("somewhere")));
            assert_eq!(message.command.as_slice(), "PING");
            assert_eq!(message.args.len(), 2);
            assert_eq!(message.args[0].as_slice(), "server1");
            assert_eq!(message.args[1].as_slice(), "server2");
        },
        Err(_) => panic!("failed to parse")
    };

    match IrcMessage::from_str(":somewhere PING :server1 server2") {
        Ok(message) => {
            assert_eq!(message.prefix_raw, Some(String::from_str("somewhere")));
            assert_eq!(message.command.as_slice(), "PING");
            assert_eq!(message.args.len(), 1);
            assert_eq!(message.args[0].as_slice(), "server1 server2");
        },
        Err(_) => panic!("failed to parse")
    };
}


#[test]
fn test_irc_message_numerics() {
    match IrcMessage::from_str(":somewhere 001 nick :blah blah") {
        Ok(message) => {
            assert_eq!(message.prefix_raw, Some(String::from_str("somewhere")));
            assert_eq!(message.command.as_slice(), "001");
            match message.message {
                IrcProtocolMessage::Numeric(num, _) => assert_eq!(num, 1),
                _ => panic!("numbers should parse as numerics")
            }

        },
        Err(_) => panic!("failed to parse")
    };

    match IrcMessage::from_str("001 nick :blah blah") {
        Ok(message) => {
            assert_eq!(message.prefix_raw, None);
            assert_eq!(message.command.as_slice(), "001");
            match message.message {
                IrcProtocolMessage::Numeric(num, _) => assert_eq!(num, 1),
                _ => panic!("numbers should parse as numerics")
            }

        },
        Err(_) => panic!("failed to parse")
    };

    match IrcMessage::from_str("366 arg") {
        Ok(message) => {
            match message.message {
                IrcProtocolMessage::Numeric(num, _) => assert_eq!(num, 366),
                _ => panic!("numbers should parse as numerics")
            }

        },
        Err(_) => panic!("failed to parse")
    };
}
