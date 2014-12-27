use std::str;
use std::string::String;
use std::fmt;
use parse::{
    IrcMsg,
    IrcMsgPrefix,
    ParseError,
    is_channel,
    can_target_channel,
};
use message_types::server;

#[deriving(Clone)]
pub struct IrcMessage {
    msg2: server::IncomingMsg,
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
    pub fn from_str(text: &str) -> Result<IrcMessage, String> {
        if text.len() == 0 {
            return Err("Invalid IRC message; empty".to_string());
        }
        
        let (_, args) = {
            if text.char_at(0) == ':' {
                let parts: Vec<&str> = text.splitn(1, ' ').collect();
                if parts.len() < 2 {
                    return Err("Invalid IRC message".to_string());
                }
                let (_, args) = match parse_message_rest(parts[1]) {
                    Ok(result) => result,
                    Err(err) => return Err(format!("Invalid IRC message: {}", err))
                };
                (Some(String::from_str(parts[0].slice_from(1))), args)
            } else {
                assert!(text.len() > 0);
                let (_, args) = match parse_message_rest(text) {
                    Ok(result) => result,
                    Err(err) => return Err(format!("Invalid IRC message: {}", err))
                };
                (None, args)
            }
        };

        let msg2 = match IrcMsg::new(text.to_string().into_bytes()) {
            Ok(msg) => server::IncomingMsg::from_msg(msg),
            Err(ParseError::InvalidMessage(desc)) => return Err(desc.to_string()),
            Err(ParseError::EncodingError) => return Err("bad encoding".to_string())
        };

        Ok(IrcMessage {
            msg2: msg2,
            args: args
        })
    }

    pub fn get_typed_message(&self) -> &server::IncomingMsg {
        &self.msg2
    }

    pub fn as_irc_msg(&self) -> &IrcMsg {
        self.msg2.to_irc_msg()
    }

    #[inline]
    pub fn is_privmsg(&self) -> bool {
        self.get_typed_message().is_privmsg()
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
    #[experimental]
    pub fn source_nick<'a>(&'a self) -> Option<String> {
        self.get_prefix().and_then(|pref| {
            match pref.nick() {
                Some(nick) => Some(nick.to_string()),
                None => None
            }
        })
    }

    pub fn get_prefix<'a>(&'a self) -> Option<IrcMsgPrefix<'a>> {
        let msg = self.get_typed_message().to_irc_msg();
        match msg.has_prefix() {    
            true => Some(msg.get_prefix()),
            false => None
        }
    }

    pub fn get_prefix_raw<'a>(&'a self) -> Option<&'a str> {
        let msg = self.get_typed_message().to_irc_msg();
        match msg.has_prefix() {
            true => Some(msg.get_prefix_str()),
            false => None
        }
    }

    #[inline]
    pub fn command<'a>(&'a self) -> &'a str {
        self.get_typed_message().to_irc_msg().get_command()
    }

    #[inline]
    #[deprecated]
    pub fn get_args<'a>(&'a self) -> Vec<&'a str> {
        let irc_msg = self.get_typed_message().to_irc_msg();

        let mut vecout = Vec::new();
        for arg in irc_msg.get_args().into_iter() {
            match str::from_utf8(arg) {
                Ok(slice) => vecout.push(slice),
                Err(_) => panic!("Bad message")
            }
        }
        vecout
    }

    #[inline]
    #[unstable]
    pub fn get_args_checked<'a>(&'a self) -> Option<Vec<&'a str>> {
        let irc_msg = self.get_typed_message().to_irc_msg();

        let mut vecout = Vec::new();
        for arg in irc_msg.get_args().into_iter() {
            match str::from_utf8(arg) {
                Ok(slice) => vecout.push(slice),
                Err(_) => return None
            }
        }
        Some(vecout)
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

        match self.get_prefix_raw() {
            Some(ref prefix) => write!(f, "IrcMessage({}, {}, {})",
                prefix.as_slice(), self.command(), arg_string.as_slice()),
            None => write!(f, "IrcMessage({}, {})",
                self.command(), arg_string.as_slice())
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

    match IrcMessage::from_str("PING server1") {
        Ok(message) => {
            assert_eq!(message.get_prefix_raw(), None);
            assert_eq!(message.command(), "PING");
            assert_eq!(message.args.len(), 1);
        },
        Err(_) => panic!("failed to parse")
    };


    match IrcMessage::from_str("PING server1 server2") {
        Ok(message) => {
            assert_eq!(message.get_prefix_raw(), None);
            assert_eq!(message.command(), "PING");
            assert_eq!(message.args.len(), 2);
        },
        Err(_) => panic!("failed to parse")
    };

    match IrcMessage::from_str(":somewhere PING server1") {
        Ok(message) => {
            assert_eq!(message.get_prefix_raw(), Some("somewhere"));
            assert_eq!(message.command(), "PING");
            assert_eq!(message.args.len(), 1);
        },
        Err(_) => panic!("failed to parse")
    };
    
    match IrcMessage::from_str(":somewhere PING server1 server2") {
        Ok(message) => {
            assert_eq!(message.get_prefix_raw(), Some("somewhere"));
            assert_eq!(message.command(), "PING");
            assert_eq!(message.args.len(), 2);
            assert_eq!(message.args[0].as_slice(), "server1");
            assert_eq!(message.args[1].as_slice(), "server2");
        },
        Err(_) => panic!("failed to parse")
    };

    match IrcMessage::from_str(":somewhere PING :server1 server2") {
        Ok(message) => {
            assert_eq!(message.get_prefix_raw(), Some("somewhere"));
            assert_eq!(message.command(), "PING");
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
            assert_eq!(message.get_prefix_raw(), Some("somewhere"));
            assert_eq!(message.command(), "001");
        },
        Err(_) => panic!("failed to parse")
    };

    match IrcMessage::from_str("001 nick :blah blah") {
        Ok(message) => {
            assert_eq!(message.get_prefix_raw(), None);
            assert_eq!(message.command(), "001");
        },
        Err(_) => panic!("failed to parse")
    };
}
